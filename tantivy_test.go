package tantivy_go_test

import (
	"encoding/base64"
	"encoding/json"
	"os"
	"testing"

	"github.com/anyproto/tantivy-go"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

const NameBody = "body"
const NameId = "id"
const NameTitle = "title"

const limit = 40
const minGram = 2

type DocSample struct {
	Title      string
	Id         string
	Body       string
	Highlights []Highlight
}

type Fragment struct {
	R [][2]int `json:"r"`
	T string   `json:"t"`
}

type Highlight struct {
	FieldName string   `json:"field_name"`
	Fragment  Fragment `json:"fragment"`
}

func Test(t *testing.T) {

	t.Run("docs search and remove - when by raw Id", func(t *testing.T) {
		schema, index := fx(t, limit, minGram, false)

		defer index.Free()

		doc, err := addDoc(t, "Example Title", "Example body doing.", "1", index)
		require.NoError(t, err)

		err = index.AddAndConsumeDocuments(doc)
		require.NoError(t, err)

		result, err := index.Search("body", 100, true, NameBody)
		require.NoError(t, err)

		size, err := result.GetSize()
		require.Equal(t, 1, int(size))

		results, err := tantivy_go.GetSearchResults(result, schema, func(jsonStr string) (interface{}, error) {
			var doc DocSample
			return doc, json.Unmarshal([]byte(jsonStr), &doc)
		}, NameId, NameTitle, NameBody)
		require.NoError(t, err)

		require.Equal(t, len(results), int(size))
		require.NoError(t, err)

		for next := range results {
			model := results[next].(DocSample)
			require.Equal(t, DocSample{
				"Example Title",
				"1",
				"Example body doing.",
				[]Highlight{
					{
						NameBody,
						Fragment{
							[][2]int{{8, 12}},
							base64.StdEncoding.EncodeToString([]byte("Example body doing")),
						},
					}},
			},
				model)
		}

		docs, err := index.NumDocs()
		require.NoError(t, err)
		require.Equal(t, uint64(1), docs)

		err = index.DeleteDocuments(NameId, "1")
		require.NoError(t, err)
		docs, err = index.NumDocs()
		require.Equal(t, uint64(0), docs)
	})

	t.Run("docs remove - when by body token", func(t *testing.T) {
		_, index := fx(t, limit, minGram, false)

		defer index.Free()

		// Tokens are strang, text, bodi
		doc, err := addDoc(t, "Example title", "Strange/text/body", "1", index)
		doc2, err := addDoc(t, "Example title", "Strange another something", "2", index)
		require.NoError(t, err)

		err = index.AddAndConsumeDocuments(doc, doc2)
		require.NoError(t, err)

		docs, err := index.NumDocs()
		require.NoError(t, err)
		require.Equal(t, uint64(2), docs)

		err = index.DeleteDocuments(NameBody, "strang")
		require.NoError(t, err)
		docs, err = index.NumDocs()
		require.Equal(t, uint64(0), docs)
	})

	t.Run("docs remove - when by wrong body token", func(t *testing.T) {
		_, index := fx(t, limit, minGram, false)

		defer index.Free()

		// Tokens are strang, text, bodi
		doc, err := addDoc(t, "Example title", "Strange/text/body", "1", index)
		doc2, err := addDoc(t, "Example title", "Strange another something", "2", index)
		require.NoError(t, err)

		err = index.AddAndConsumeDocuments(doc, doc2)
		require.NoError(t, err)

		docs, err := index.NumDocs()
		require.NoError(t, err)
		require.Equal(t, uint64(2), docs)

		err = index.DeleteDocuments(NameBody, "strange")
		require.NoError(t, err)
		docs, err = index.NumDocs()
		require.Equal(t, uint64(2), docs)
	})

	t.Run("docs remove - when by proper token and wrong field length", func(t *testing.T) {
		_, index := fx(t, 1, minGram, false)

		defer index.Free()

		doc, err := addDoc(t, "Example title", "Body", "12", index)
		require.NoError(t, err)

		err = index.AddAndConsumeDocuments(doc)
		require.NoError(t, err)

		docs, err := index.NumDocs()
		require.NoError(t, err)
		require.Equal(t, uint64(1), docs)

		// token limit is 1
		err = index.DeleteDocuments(NameBody, "12")
		require.NoError(t, err)
		docs, err = index.NumDocs()
		require.Equal(t, uint64(1), docs)
	})

	t.Run("docs search and remove - when thai", func(t *testing.T) {
		_, index := fx(t, limit, 1, false)

		defer index.Free()

		doc, err := addDoc(t, "ตัวอย่ง", "body", "1", index)
		require.NoError(t, err)

		err = index.AddAndConsumeDocuments(doc)
		require.NoError(t, err)

		docs, err := index.NumDocs()
		require.NoError(t, err)
		require.Equal(t, uint64(1), docs)

		result, err := index.Search("ย", 100, true, NameTitle)
		require.NoError(t, err)

		size, err := result.GetSize()
		defer result.Free()
		require.Equal(t, 0, int(size))

		result2, err := index.Search("ย่", 100, true, NameTitle)
		require.NoError(t, err)

		size2, err := result2.GetSize()
		defer result2.Free()
		require.Equal(t, 1, int(size2))

		err = index.DeleteDocuments(NameTitle, "ต")
		require.NoError(t, err)
		docs, err = index.NumDocs()
		require.Equal(t, uint64(1), docs)

		err = index.DeleteDocuments(NameTitle, "ตั")
		require.NoError(t, err)
		docs, err = index.NumDocs()
		require.Equal(t, uint64(0), docs)
	})

	t.Run("docs search and remove - when fast", func(t *testing.T) {
		_, index := fx(t, limit, minGram, false)

		defer index.Free()

		doc, err := addDoc(t, "some", "body", "1", index)
		require.NoError(t, err)

		err = index.AddAndConsumeDocuments(doc)
		require.NoError(t, err)

		docs, err := index.NumDocs()
		require.NoError(t, err)
		require.Equal(t, uint64(1), docs)

		result, err := index.Search("1", 100, true, NameId)
		require.NoError(t, err)

		size, err := result.GetSize()
		defer result.Free()
		require.Equal(t, 1, int(size))

		err = index.DeleteDocuments(NameId, "1")
		docs, err = index.NumDocs()
		require.NoError(t, err)
		require.Equal(t, uint64(0), docs)
	})

	t.Run("docs search and remove - when title", func(t *testing.T) {
		schema, index := fx(t, limit, minGram, false)

		defer index.Free()

		doc, err := addDoc(t, "Create Body", "Example title content.", "1", index)
		require.NoError(t, err)

		err = index.AddAndConsumeDocuments(doc)
		require.NoError(t, err)

		result, err := index.Search("create", 100, true, NameTitle)
		require.NoError(t, err)

		size, err := result.GetSize()
		require.Equal(t, 1, int(size))

		results, err := tantivy_go.GetSearchResults(result, schema, func(jsonStr string) (interface{}, error) {
			var doc DocSample
			return doc, json.Unmarshal([]byte(jsonStr), &doc)
		}, NameId, NameTitle, NameBody)
		require.NoError(t, err)

		require.Equal(t, len(results), int(size))
		require.NoError(t, err)

		for next := range results {
			model := results[next].(DocSample)
			require.Equal(t, DocSample{
				"Create Body",
				"1",
				"Example title content.",
				[]Highlight{
					{
						NameTitle,
						Fragment{
							[][2]int{{0, 2}, {0, 3}, {0, 4}},
							base64.StdEncoding.EncodeToString([]byte("Crea")),
						},
					}},
			},
				model)
		}

		docs, err := index.NumDocs()
		require.NoError(t, err)
		require.Equal(t, uint64(1), docs)

		err = index.DeleteDocuments(NameId, "1")
		require.NoError(t, err)
		docs, err = index.NumDocs()
		require.Equal(t, uint64(0), docs)
	})
}

func addDoc(
	t *testing.T,
	title string,
	name string,
	id string,
	index *tantivy_go.Index,
) (*tantivy_go.Document, error) {
	doc := tantivy_go.NewDocument()

	err := doc.AddField(NameTitle, title, index)
	require.NoError(t, err)

	err = doc.AddField(NameId, id, index)
	require.NoError(t, err)

	err = doc.AddField(NameBody, name, index)
	return doc, err
}

func fx(
	t *testing.T,
	limit uintptr,
	minGram uintptr,
	isFastId bool,
) (*tantivy_go.Schema, *tantivy_go.Index) {
	err := tantivy_go.LibInit("release")
	assert.NoError(t, err)
	builder, err := tantivy_go.NewSchemaBuilder()
	require.NoError(t, err)

	err = builder.AddTextField(
		NameTitle,
		true,
		true,
		false,
		tantivy_go.IndexRecordOptionWithFreqsAndPositions,
		tantivy_go.TokenizerEdgeNgram,
	)
	require.NoError(t, err)

	err = builder.AddTextField(
		NameId,
		true,
		false,
		isFastId,
		tantivy_go.IndexRecordOptionBasic,
		tantivy_go.TokenizerRaw,
	)
	require.NoError(t, err)

	err = builder.AddTextField(
		NameBody,
		true,
		true,
		false,
		tantivy_go.IndexRecordOptionWithFreqsAndPositions,
		tantivy_go.TokenizerSimple,
	)
	require.NoError(t, err)

	schema, err := builder.BuildSchema()
	require.NoError(t, err)

	os.RemoveAll("index_dir")
	index, err := tantivy_go.NewIndexWithSchema("index_dir", schema)
	require.NoError(t, err)

	err = index.RegisterTextAnalyzerSimple(tantivy_go.TokenizerSimple, limit, tantivy_go.English)
	require.NoError(t, err)

	err = index.RegisterTextAnalyzerEdgeNgram(tantivy_go.TokenizerEdgeNgram, minGram, 4, 100)
	require.NoError(t, err)

	err = index.RegisterTextAnalyzerRaw(tantivy_go.TokenizerRaw)
	require.NoError(t, err)
	return schema, index
}
