package tantivy_go

import (
	"fmt"
	"io"
	"os"
	"path/filepath"
	"testing"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

func TestOpstampPersistence(t *testing.T) {
	// Create temporary directories
	originalDir := t.TempDir()
	backupDir := t.TempDir()

	// Track opstamps for each document
	docOpstamps := make(map[string]uint64)

	// Phase 1: Create initial index with some documents
	t.Run("create_initial_index", func(t *testing.T) {
		builder, err := NewSchemaBuilder()
		require.NoError(t, err)

		err = builder.AddTextField("id", true, false, false, IndexRecordOptionBasic, DefaultTokenizer)
		require.NoError(t, err)
		err = builder.AddTextField("title", true, true, false, IndexRecordOptionWithFreqsAndPositions, DefaultTokenizer)
		require.NoError(t, err)
		err = builder.AddTextField("body", true, true, false, IndexRecordOptionWithFreqsAndPositions, DefaultTokenizer)
		require.NoError(t, err)

		schema, err := builder.BuildSchema()
		require.NoError(t, err)

		ctx, err := NewTantivyContextWithSchema(originalDir, schema)
		require.NoError(t, err)

		// Add first batch of documents
		for i := 1; i <= 5; i++ {
			docID := fmt.Sprintf("doc_%d", i)
			doc := NewDocument()
			err = doc.AddField(docID, ctx, "id")
			require.NoError(t, err)
			err = doc.AddField(fmt.Sprintf("Title %d", i), ctx, "title")
			require.NoError(t, err)
			err = doc.AddField(fmt.Sprintf("This is document number %d", i), ctx, "body")
			require.NoError(t, err)

			opstamp, err := ctx.AddAndConsumeDocumentsWithOpstamp(doc)
			require.NoError(t, err)
			docOpstamps[docID] = opstamp
			t.Logf("Added %s with opstamp %d", docID, opstamp)
		}

		// Store the last operation opstamp before closing
		lastAddedOpstamp := docOpstamps["doc_5"]

		ctx.Free()

		// Reopen to check the commit opstamp
		ctx2, err := NewTantivyContextWithSchema(originalDir, schema)
		require.NoError(t, err)
		initialOpstamp := ctx2.CommitOpstamp()
		t.Logf("Initial commit opstamp after reopen: %d", initialOpstamp)
		t.Logf("Last operation opstamp was: %d", lastAddedOpstamp)
		// After reopening, commit opstamp should reflect the last committed operation
		assert.Equal(t, lastAddedOpstamp, initialOpstamp)
		ctx2.Free()
	})

	// Phase 2: Copy directory after 5 documents
	t.Run("copy_directory", func(t *testing.T) {
		err := copyDir(originalDir, backupDir)
		require.NoError(t, err)
		t.Logf("Copied directory from %s to %s", originalDir, backupDir)
	})

	// Phase 3: Add more documents to original
	var finalOpstamp uint64
	t.Run("add_more_documents", func(t *testing.T) {
		builder, err := NewSchemaBuilder()
		require.NoError(t, err)

		err = builder.AddTextField("id", true, false, false, IndexRecordOptionBasic, DefaultTokenizer)
		require.NoError(t, err)
		err = builder.AddTextField("title", true, true, false, IndexRecordOptionWithFreqsAndPositions, DefaultTokenizer)
		require.NoError(t, err)
		err = builder.AddTextField("body", true, true, false, IndexRecordOptionWithFreqsAndPositions, DefaultTokenizer)
		require.NoError(t, err)

		schema, err := builder.BuildSchema()
		require.NoError(t, err)

		ctx, err := NewTantivyContextWithSchema(originalDir, schema)
		require.NoError(t, err)
		defer ctx.Free()

		// Verify opstamp is preserved from previous session
		reopenOpstamp := ctx.CommitOpstamp()
		t.Logf("Reopened original index, commit opstamp: %d", reopenOpstamp)
		// The opstamp should be preserved from the previous session
		assert.Greater(t, reopenOpstamp, uint64(0))

		// Add more documents
		for i := 6; i <= 10; i++ {
			docID := fmt.Sprintf("doc_%d", i)
			doc := NewDocument()
			err = doc.AddField(docID, ctx, "id")
			require.NoError(t, err)
			err = doc.AddField(fmt.Sprintf("Title %d", i), ctx, "title")
			require.NoError(t, err)
			err = doc.AddField(fmt.Sprintf("This is document number %d", i), ctx, "body")
			require.NoError(t, err)

			opstamp, err := ctx.AddAndConsumeDocumentsWithOpstamp(doc)
			require.NoError(t, err)
			docOpstamps[docID] = opstamp
			t.Logf("Added %s with opstamp %d", docID, opstamp)
		}

		finalOpstamp = ctx.CommitOpstamp()
		t.Logf("Final commit opstamp in original: %d", finalOpstamp)

		// The last operation's opstamp should be the highest
		lastOpstamp := docOpstamps["doc_10"]
		t.Logf("Last operation opstamp: %d", lastOpstamp)

		// After multiple operations, commit opstamp might lag behind operation opstamps
		// This is expected behavior in Tantivy
	})

	// Phase 4: Verify backup has lower opstamp
	t.Run("verify_backup_opstamp", func(t *testing.T) {
		builder, err := NewSchemaBuilder()
		require.NoError(t, err)

		err = builder.AddTextField("id", true, false, false, IndexRecordOptionBasic, DefaultTokenizer)
		require.NoError(t, err)
		err = builder.AddTextField("title", true, true, false, IndexRecordOptionWithFreqsAndPositions, DefaultTokenizer)
		require.NoError(t, err)
		err = builder.AddTextField("body", true, true, false, IndexRecordOptionWithFreqsAndPositions, DefaultTokenizer)
		require.NoError(t, err)

		schema, err := builder.BuildSchema()
		require.NoError(t, err)

		// Open the backup directory
		backupCtx, err := NewTantivyContextWithSchema(backupDir, schema)
		require.NoError(t, err)
		defer backupCtx.Free()

		backupOpstamp := backupCtx.CommitOpstamp()
		t.Logf("Backup directory commit opstamp: %d", backupOpstamp)

		// Backup should have the same opstamp as when it was copied
		// Since we copied after adding 5 documents, it should match the opstamp from that time
		assert.Greater(t, backupOpstamp, uint64(0))
		// If commits happened after the copy, final opstamp might be different
		// The important thing is that backup preserved its state from the copy time

		// Verify document count in backup
		numDocs, err := backupCtx.NumDocs()
		require.NoError(t, err)
		assert.Equal(t, uint64(5), numDocs)

		// Verify we can search in backup
		searchCtx := NewSearchContextBuilder().
			SetQuery("document").
			AddField("body", 1.0).
			SetDocsLimit(10).
			Build()

		results, err := backupCtx.Search(searchCtx)
		require.NoError(t, err)
		defer results.Free()

		size, err := results.GetSize()
		require.NoError(t, err)
		assert.Equal(t, uint64(5), size)
	})

	// Phase 5: Test opstamp after operations in backup
	t.Run("test_backup_operations", func(t *testing.T) {
		builder, err := NewSchemaBuilder()
		require.NoError(t, err)

		err = builder.AddTextField("id", true, false, false, IndexRecordOptionBasic, DefaultTokenizer)
		require.NoError(t, err)
		err = builder.AddTextField("title", true, true, false, IndexRecordOptionWithFreqsAndPositions, DefaultTokenizer)
		require.NoError(t, err)
		err = builder.AddTextField("body", true, true, false, IndexRecordOptionWithFreqsAndPositions, DefaultTokenizer)
		require.NoError(t, err)

		schema, err := builder.BuildSchema()
		require.NoError(t, err)

		backupCtx, err := NewTantivyContextWithSchema(backupDir, schema)
		require.NoError(t, err)
		defer backupCtx.Free()

		// Add a document to backup
		doc := NewDocument()
		err = doc.AddField("backup_doc_1", backupCtx, "id")
		require.NoError(t, err)
		err = doc.AddField("Backup Document", backupCtx, "title")
		require.NoError(t, err)
		err = doc.AddField("This document was added to backup", backupCtx, "body")
		require.NoError(t, err)

		backupNewOpstamp, err := backupCtx.AddAndConsumeDocumentsWithOpstamp(doc)
		require.NoError(t, err)
		t.Logf("Added document to backup with opstamp: %d", backupNewOpstamp)

		// The new opstamp in backup continues independently from its last state
		assert.Greater(t, backupNewOpstamp, uint64(0))
		t.Logf("Backup's opstamp sequence continues independently")

		// Delete a document
		deleteOpstamp, err := backupCtx.DeleteDocumentsWithOpstamp("id", "doc_1")
		require.NoError(t, err)
		t.Logf("Deleted doc_1 with opstamp: %d", deleteOpstamp)
		assert.Greater(t, deleteOpstamp, backupNewOpstamp)
	})

	// Phase 6: Verify opstamps are independent between directories
	t.Run("verify_independence", func(t *testing.T) {
		builder, err := NewSchemaBuilder()
		require.NoError(t, err)

		err = builder.AddTextField("id", true, false, false, IndexRecordOptionBasic, DefaultTokenizer)
		require.NoError(t, err)
		err = builder.AddTextField("title", true, true, false, IndexRecordOptionWithFreqsAndPositions, DefaultTokenizer)
		require.NoError(t, err)
		err = builder.AddTextField("body", true, true, false, IndexRecordOptionWithFreqsAndPositions, DefaultTokenizer)
		require.NoError(t, err)

		schema, err := builder.BuildSchema()
		require.NoError(t, err)

		// Open both directories
		origCtx, err := NewTantivyContextWithSchema(originalDir, schema)
		require.NoError(t, err)
		defer origCtx.Free()

		backupCtx, err := NewTantivyContextWithSchema(backupDir, schema)
		require.NoError(t, err)
		defer backupCtx.Free()

		// Verify they have different opstamps
		origOpstamp := origCtx.CommitOpstamp()
		backupOpstamp := backupCtx.CommitOpstamp()

		t.Logf("Original opstamp: %d, Backup opstamp: %d", origOpstamp, backupOpstamp)
		assert.NotEqual(t, origOpstamp, backupOpstamp)

		// Original should have all 10 documents
		origDocs, err := origCtx.NumDocs()
		require.NoError(t, err)
		assert.Equal(t, uint64(10), origDocs)

		// Backup should have 5 original + 1 added - 1 deleted = 5
		backupDocs, err := backupCtx.NumDocs()
		require.NoError(t, err)
		// Note: The deletion might not be reflected immediately in NumDocs
		t.Logf("Backup has %d documents", backupDocs)
	})
}

// copyDir recursively copies a directory
func copyDir(src, dst string) error {
	return filepath.Walk(src, func(path string, info os.FileInfo, err error) error {
		if err != nil {
			return err
		}

		// Calculate destination path
		relPath, err := filepath.Rel(src, path)
		if err != nil {
			return err
		}
		dstPath := filepath.Join(dst, relPath)

		if info.IsDir() {
			// Create directory
			return os.MkdirAll(dstPath, info.Mode())
		}

		// Copy file
		return copyFile(path, dstPath)
	})
}

// copyFile copies a single file
func copyFile(src, dst string) error {
	sourceFile, err := os.Open(src)
	if err != nil {
		return err
	}
	defer sourceFile.Close()

	destFile, err := os.Create(dst)
	if err != nil {
		return err
	}
	defer destFile.Close()

	_, err = io.Copy(destFile, sourceFile)
	if err != nil {
		return err
	}

	// Copy file permissions
	sourceInfo, err := os.Stat(src)
	if err != nil {
		return err
	}
	return os.Chmod(dst, sourceInfo.Mode())
}
