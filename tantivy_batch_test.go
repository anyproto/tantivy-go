package tantivy_go

import (
	"fmt"
	"os"
	"path/filepath"
	"testing"

	"github.com/stretchr/testify/require"
)

func TestBatchAddAndDeleteDocuments(t *testing.T) {
	tempDir := t.TempDir()
	indexPath := filepath.Join(tempDir, "test-batch-index")
	defer os.RemoveAll(indexPath)

	// Create schema
	builder, err := NewSchemaBuilder()
	require.NoError(t, err)

	err = builder.AddTextField("id", true, false, false, IndexRecordOptionBasic, "simple")
	require.NoError(t, err)
	err = builder.AddTextField("body", true, true, false, IndexRecordOptionWithFreqsAndPositions, "simple")
	require.NoError(t, err)

	schema, err := builder.BuildSchema()
	require.NoError(t, err)

	// Create index
	index, err := NewTantivyContextWithSchema(indexPath, schema)
	require.NoError(t, err)
	defer index.Free()

	// Register tokenizer
	err = index.RegisterTextAnalyzerSimple("simple", 100, English)
	require.NoError(t, err)

	// Test 1: Add documents only
	t.Run("Add documents only", func(t *testing.T) {
		doc1 := NewDocument()
		err := doc1.AddField("1", index, "id")
		require.NoError(t, err)
		err = doc1.AddField("first document", index, "body")
		require.NoError(t, err)

		doc2 := NewDocument()
		err = doc2.AddField("2", index, "id")
		require.NoError(t, err)
		err = doc2.AddField("second document", index, "body")
		require.NoError(t, err)

		docs := []*Document{doc1, doc2}

		opstamp, err := index.BatchAddAndDeleteDocumentsWithOpstamp(docs, "", nil)
		require.NoError(t, err)
		require.Greater(t, opstamp, uint64(0))

		numDocs, err := index.NumDocs()
		require.NoError(t, err)
		require.Equal(t, uint64(2), numDocs)
	})

	// Test 2: Delete documents only
	t.Run("Delete documents only", func(t *testing.T) {
		deleteFieldValues := []string{"1"}

		opstamp, err := index.BatchAddAndDeleteDocumentsWithOpstamp(nil, "id", deleteFieldValues)
		require.NoError(t, err)
		require.Greater(t, opstamp, uint64(0))

		numDocs, err := index.NumDocs()
		require.NoError(t, err)
		require.Equal(t, uint64(1), numDocs)
	})

	// Test 3: Add and delete in single commit
	t.Run("Add and delete in single commit", func(t *testing.T) {
		// Add documents 3 and 4, delete document 2
		doc3 := NewDocument()
		err = doc3.AddField("3", index, "id")
		require.NoError(t, err)
		err = doc3.AddField("third document", index, "body")
		require.NoError(t, err)

		doc4 := NewDocument()
		err = doc4.AddField("4", index, "id")
		require.NoError(t, err)
		err = doc4.AddField("fourth document", index, "body")
		require.NoError(t, err)

		addDocs := []*Document{doc3, doc4}
		deleteFieldValues := []string{"2"}

		opstamp, err := index.BatchAddAndDeleteDocumentsWithOpstamp(addDocs, "id", deleteFieldValues)
		require.NoError(t, err)
		require.Greater(t, opstamp, uint64(0))

		numDocs, err := index.NumDocs()
		require.NoError(t, err)
		require.Equal(t, uint64(2), numDocs) // Should have docs 3 and 4

		// Verify correct documents remain
		sCtx := NewSearchContextBuilder().
			SetQuery("third OR fourth").
			AddFieldDefaultWeight("body").
			SetDocsLimit(10).
			Build()
		results, err := index.Search(sCtx)
		require.NoError(t, err)
		defer results.Free()

		size, err := results.GetSize()
		require.NoError(t, err)
		require.Equal(t, uint64(2), uint64(size))
	})

	// Test 4: Empty batch operation
	t.Run("Empty batch operation", func(t *testing.T) {
		opstamp, err := index.BatchAddAndDeleteDocumentsWithOpstamp(nil, "", nil)
		require.NoError(t, err)
		require.Equal(t, uint64(0), opstamp) // Should return 0 for empty operation
	})

	// Test 5: Invalid delete field
	t.Run("Invalid delete field", func(t *testing.T) {
		deleteFieldValues := []string{"some_id"}

		_, err := index.BatchAddAndDeleteDocumentsWithOpstamp(nil, "nonexistent_field", deleteFieldValues)
		require.Error(t, err)
		require.Contains(t, err.Error(), "field not found in schema")
	})
}

// TestBatchOperationPerformance demonstrates the performance benefit of batch operations
func TestBatchOperationPerformance(t *testing.T) {
	tempDir := t.TempDir()
	indexPath := filepath.Join(tempDir, "test-batch-performance")
	defer os.RemoveAll(indexPath)

	// Create schema
	builder, err := NewSchemaBuilder()
	require.NoError(t, err)

	err = builder.AddTextField("id", true, false, false, IndexRecordOptionBasic, "simple")
	require.NoError(t, err)
	err = builder.AddTextField("body", true, true, false, IndexRecordOptionWithFreqsAndPositions, "simple")
	require.NoError(t, err)

	schema, err := builder.BuildSchema()
	require.NoError(t, err)

	// Create index
	index, err := NewTantivyContextWithSchema(indexPath, schema)
	require.NoError(t, err)
	defer index.Free()

	// Register tokenizer
	err = index.RegisterTextAnalyzerSimple("simple", 100, English)
	require.NoError(t, err)

	// Prepare test data
	numDocs := 100
	addDocs := make([]*Document, numDocs)
	for i := 0; i < numDocs; i++ {
		doc := NewDocument()
		err = doc.AddField(fmt.Sprintf("doc_%d", i), index, "id")
		require.NoError(t, err)
		err = doc.AddField(fmt.Sprintf("This is document number %d", i), index, "body")
		require.NoError(t, err)
		addDocs[i] = doc
	}

	deleteFieldValues := make([]string, numDocs/2)
	for i := 0; i < numDocs/2; i++ {
		deleteFieldValues[i] = fmt.Sprintf("doc_%d", i*2) // Delete every even document
	}

	// Test batch operation
	opstamp, err := index.BatchAddAndDeleteDocumentsWithOpstamp(addDocs, "id", deleteFieldValues)
	require.NoError(t, err)
	require.Greater(t, opstamp, uint64(0))

	// Verify final document count
	// We're adding 100 new documents and deleting 50 that don't exist yet,
	// so we should have all 100 documents
	numDocsAfter, err := index.NumDocs()
	require.NoError(t, err)
	require.Equal(t, uint64(numDocs), numDocsAfter)

	t.Logf("Successfully performed batch operation: added %d docs, deleted %d docs in single commit",
		numDocs, len(deleteFieldValues))
}

// TestBatchDeleteAndAddSameID tests the important case of deleting and adding the same ID in a single batch
// This is a common pattern for updating documents
func TestBatchDeleteAndAddSameID(t *testing.T) {
	tempDir := t.TempDir()
	indexPath := filepath.Join(tempDir, "test-batch-update")
	defer os.RemoveAll(indexPath)

	// Create simple schema
	builder, err := NewSchemaBuilder()
	require.NoError(t, err)

	err = builder.AddTextField("id", true, false, false, IndexRecordOptionBasic, "raw")
	require.NoError(t, err)
	err = builder.AddTextField("content", true, true, false, IndexRecordOptionWithFreqsAndPositions, "simple")
	require.NoError(t, err)

	schema, err := builder.BuildSchema()
	require.NoError(t, err)

	// Create index
	index, err := NewTantivyContextWithSchema(indexPath, schema)
	require.NoError(t, err)
	defer index.Free()

	// Register tokenizers
	err = index.RegisterTextAnalyzerSimple("simple", 100, English)
	require.NoError(t, err)
	err = index.RegisterTextAnalyzerRaw("raw")
	require.NoError(t, err)

	// Add initial document
	t.Run("Setup initial document", func(t *testing.T) {
		doc := NewDocument()
		err = doc.AddField("doc1", index, "id")
		require.NoError(t, err)
		err = doc.AddField("Initial content", index, "content")
		require.NoError(t, err)

		_, err = index.AddAndConsumeDocumentsWithOpstamp(doc)
		require.NoError(t, err)

		count, err := index.NumDocs()
		require.NoError(t, err)
		require.Equal(t, uint64(1), count)
	})

	// Test delete and add same ID in one batch (update pattern)
	t.Run("Delete and add same ID in batch", func(t *testing.T) {
		// Create new document with same ID but different content
		doc := NewDocument()
		err = doc.AddField("doc1", index, "id")
		require.NoError(t, err)
		err = doc.AddField("Updated content", index, "content")
		require.NoError(t, err)

		// Delete and add in same batch
		deleteFieldValues := []string{"doc1"}
		opstamp, err := index.BatchAddAndDeleteDocumentsWithOpstamp([]*Document{doc}, "id", deleteFieldValues)
		require.NoError(t, err)
		require.Greater(t, opstamp, uint64(0))

		// Check document count - should still be 1
		count, err := index.NumDocs()
		require.NoError(t, err)
		require.Equal(t, uint64(1), count, "Should still have 1 document after update")

		// Verify content was updated
		sCtx := NewSearchContextBuilder().
			SetQuery("doc1").
			AddFieldDefaultWeight("id").
			SetDocsLimit(1).
			Build()

		results, err := index.Search(sCtx)
		require.NoError(t, err)

		size, err := results.GetSize()
		require.NoError(t, err)
		require.Equal(t, uint64(1), uint64(size), "Should find the document")

		if size > 0 {
			doc, err := results.Get(0)
			require.NoError(t, err)

			jsonStr, err := doc.ToJson(index, "id", "content")
			require.NoError(t, err)
			doc.Free()

			require.Contains(t, jsonStr, "Updated content", "Content should be updated")
		}
		results.Free()
	})

	// Test multiple updates in sequence
	t.Run("Multiple updates in sequence", func(t *testing.T) {
		// Update the document 3 times
		contents := []string{"First update", "Second update", "Third update"}

		for i, content := range contents {
			doc := NewDocument()
			err = doc.AddField("doc1", index, "id")
			require.NoError(t, err)
			err = doc.AddField(content, index, "content")
			require.NoError(t, err)

			deleteFieldValues := []string{"doc1"}
			opstamp, err := index.BatchAddAndDeleteDocumentsWithOpstamp([]*Document{doc}, "id", deleteFieldValues)
			require.NoError(t, err)
			require.Greater(t, opstamp, uint64(0), "Update %d should return valid opstamp", i+1)
		}

		// Verify final state
		count, err := index.NumDocs()
		require.NoError(t, err)
		require.Equal(t, uint64(1), count, "Should still have exactly 1 document")

		// Check final content
		sCtx := NewSearchContextBuilder().
			SetQuery("\"Third update\"").
			AddFieldDefaultWeight("content").
			SetDocsLimit(1).
			Build()

		results, err := index.Search(sCtx)
		require.NoError(t, err)
		size, err := results.GetSize()
		require.NoError(t, err)
		require.Equal(t, uint64(1), uint64(size), "Should find document with final content")
		results.Free()
	})
}
