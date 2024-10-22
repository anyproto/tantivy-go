package tantivy_go_test

import (
	"encoding/base64"
	"encoding/json"
	"os"
	"testing"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"

	"github.com/anyproto/tantivy-go"
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
		schema, tc := fx(t, limit, minGram, false)

		defer tc.Free()

		doc, err := addDoc(t, "Example Title", "Example body doing.", "1", tc)
		require.NoError(t, err)

		err = tc.AddAndConsumeDocuments(doc)
		require.NoError(t, err)

		result, err := tc.Search("body", 100, true, NameBody)
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

		docs, err := tc.NumDocs()
		require.NoError(t, err)
		require.Equal(t, uint64(1), docs)

		err = tc.DeleteDocuments(NameId, "1")
		require.NoError(t, err)
		docs, err = tc.NumDocs()
		require.Equal(t, uint64(0), docs)
	})

	t.Run("docs remove - when by body token", func(t *testing.T) {
		_, tc := fx(t, limit, minGram, false)

		defer tc.Free()

		// Tokens are strang, text, bodi
		doc, err := addDoc(t, "Example title", "Strange/text/body", "1", tc)
		doc2, err := addDoc(t, "Example title", "Strange another something", "2", tc)
		require.NoError(t, err)

		err = tc.AddAndConsumeDocuments(doc, doc2)
		require.NoError(t, err)

		docs, err := tc.NumDocs()
		require.NoError(t, err)
		require.Equal(t, uint64(2), docs)

		err = tc.DeleteDocuments(NameBody, "strang")
		require.NoError(t, err)
		docs, err = tc.NumDocs()
		require.Equal(t, uint64(0), docs)
	})

	t.Run("docs remove - when by wrong body token", func(t *testing.T) {
		_, tc := fx(t, limit, minGram, false)

		defer tc.Free()

		// Tokens are strang, text, bodi
		doc, err := addDoc(t, "Example title", "Strange/text/body", "1", tc)
		doc2, err := addDoc(t, "Example title", "Strange another something", "2", tc)
		require.NoError(t, err)

		err = tc.AddAndConsumeDocuments(doc, doc2)
		require.NoError(t, err)

		docs, err := tc.NumDocs()
		require.NoError(t, err)
		require.Equal(t, uint64(2), docs)

		err = tc.DeleteDocuments(NameBody, "strange")
		require.NoError(t, err)
		docs, err = tc.NumDocs()
		require.Equal(t, uint64(2), docs)
	})

	t.Run("docs remove - when by proper token and wrong field length", func(t *testing.T) {
		_, tc := fx(t, 1, minGram, false)

		defer tc.Free()

		doc, err := addDoc(t, "Example title", "Body", "12", tc)
		require.NoError(t, err)

		err = tc.AddAndConsumeDocuments(doc)
		require.NoError(t, err)

		docs, err := tc.NumDocs()
		require.NoError(t, err)
		require.Equal(t, uint64(1), docs)

		// token limit is 1
		err = tc.DeleteDocuments(NameBody, "12")
		require.NoError(t, err)
		docs, err = tc.NumDocs()
		require.Equal(t, uint64(1), docs)
	})

	t.Run("docs search and remove - when thai", func(t *testing.T) {
		_, tc := fx(t, limit, 1, false)

		defer tc.Free()

		doc, err := addDoc(t, "ตัวอย่ง", "body", "1", tc)
		require.NoError(t, err)

		err = tc.AddAndConsumeDocuments(doc)
		require.NoError(t, err)

		docs, err := tc.NumDocs()
		require.NoError(t, err)
		require.Equal(t, uint64(1), docs)

		result, err := tc.Search("ย", 100, true, NameTitle)
		require.NoError(t, err)

		size, err := result.GetSize()
		defer result.Free()
		require.Equal(t, 0, int(size))

		result2, err := tc.Search("ย่", 100, true, NameTitle)
		require.NoError(t, err)

		size2, err := result2.GetSize()
		defer result2.Free()
		require.Equal(t, 1, int(size2))

		err = tc.DeleteDocuments(NameTitle, "ต")
		require.NoError(t, err)
		docs, err = tc.NumDocs()
		require.Equal(t, uint64(1), docs)

		err = tc.DeleteDocuments(NameTitle, "ตั")
		require.NoError(t, err)
		docs, err = tc.NumDocs()
		require.Equal(t, uint64(0), docs)
	})

	t.Run("docs search - when ascii folding", func(t *testing.T) {
		_, tc := fx(t, limit, 1, false)

		defer tc.Free()

		doc, err := addDoc(t, "Idées fête", "mères straße", "1", tc)
		require.NoError(t, err)

		err = tc.AddAndConsumeDocuments(doc)
		require.NoError(t, err)

		docs, err := tc.NumDocs()
		require.NoError(t, err)
		require.Equal(t, uint64(1), docs)

		result, err := tc.Search("Idées fête", 100, true, NameTitle)
		require.NoError(t, err)

		size, err := result.GetSize()
		defer result.Free()
		require.Equal(t, 1, int(size))

		result2, err := tc.Search("idees fete", 100, true, NameTitle)
		require.NoError(t, err)

		size2, err := result2.GetSize()
		defer result2.Free()
		require.Equal(t, 1, int(size2))

		result3, err := tc.Search("straße", 100, true, NameBody)
		require.NoError(t, err)

		size3, err := result3.GetSize()
		defer result3.Free()
		require.Equal(t, 1, int(size3))

		result4, err := tc.Search("strasse", 100, true, NameBody)
		require.NoError(t, err)

		size4, err := result4.GetSize()
		defer result4.Free()
		require.Equal(t, 1, int(size4))
	})

	t.Run("docs search and remove - when fast", func(t *testing.T) {
		_, tc := fx(t, limit, minGram, false)

		defer tc.Free()

		doc, err := addDoc(t, "some", "body", "1", tc)
		require.NoError(t, err)

		err = tc.AddAndConsumeDocuments(doc)
		require.NoError(t, err)

		docs, err := tc.NumDocs()
		require.NoError(t, err)
		require.Equal(t, uint64(1), docs)

		result, err := tc.Search("1", 100, true, NameId)
		require.NoError(t, err)

		size, err := result.GetSize()
		defer result.Free()
		require.Equal(t, 1, int(size))

		err = tc.DeleteDocuments(NameId, "1")
		docs, err = tc.NumDocs()
		require.NoError(t, err)
		require.Equal(t, uint64(0), docs)
	})

	t.Run("docs search and remove - when title", func(t *testing.T) {
		schema, tc := fx(t, limit, minGram, false)

		defer tc.Free()

		doc, err := addDoc(t, "Create Body", "Example title content.", "1", tc)
		require.NoError(t, err)

		err = tc.AddAndConsumeDocuments(doc)
		require.NoError(t, err)

		result, err := tc.Search("create", 100, true, NameTitle)
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

		docs, err := tc.NumDocs()
		require.NoError(t, err)
		require.Equal(t, uint64(1), docs)

		err = tc.DeleteDocuments(NameId, "1")
		require.NoError(t, err)
		docs, err = tc.NumDocs()
		require.Equal(t, uint64(0), docs)
	})
}

func addDoc(
	t *testing.T,
	title string,
	name string,
	id string,
	tc *tantivy_go.TantivyContext,
) (*tantivy_go.Document, error) {
	doc := tantivy_go.NewDocument()

	err := doc.AddField(NameTitle, title, tc)
	require.NoError(t, err)

	err = doc.AddField(NameId, id, tc)
	require.NoError(t, err)

	err = doc.AddField(NameBody, name, tc)
	return doc, err
}

func fx(
	t *testing.T,
	limit uintptr,
	minGram uintptr,
	isFastId bool,
) (*tantivy_go.Schema, *tantivy_go.TantivyContext) {
	err := tantivy_go.LibInit(true, "debug")
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

	_ = os.RemoveAll("index_dir")
	tc, err := tantivy_go.NewTantivyContextWithSchema("index_dir", schema)
	require.NoError(t, err)

	err = tc.RegisterTextAnalyzerSimple(tantivy_go.TokenizerSimple, limit, tantivy_go.English)
	require.NoError(t, err)

	err = tc.RegisterTextAnalyzerEdgeNgram(tantivy_go.TokenizerEdgeNgram, minGram, 4, 100)
	require.NoError(t, err)

	err = tc.RegisterTextAnalyzerRaw(tantivy_go.TokenizerRaw)
	require.NoError(t, err)
	return schema, tc
}
