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
const NameBodyZh = "bodyCh"
const NameTitleZh = "titleCh"

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

		sCtx := tantivy_go.NewSearchContextBuilder().
			SetQuery("body").
			SetDocsLimit(100).
			SetWithHighlights(true).
			AddFieldDefaultWeight(NameBody).
			Build()

		result, err := tc.Search(sCtx)
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

		sCtx := tantivy_go.NewSearchContextBuilder().
			SetQuery("ย").
			SetDocsLimit(100).
			SetWithHighlights(true).
			AddFieldDefaultWeight(NameTitle).
			Build()
		result, err := tc.Search(sCtx)
		require.NoError(t, err)

		size, err := result.GetSize()
		defer result.Free()
		require.Equal(t, 0, int(size))

		sCtx2 := tantivy_go.NewSearchContextBuilder().
			SetQuery("ย่").
			SetDocsLimit(100).
			SetWithHighlights(true).
			AddFieldDefaultWeight(NameTitle).
			Build()
		result2, err := tc.Search(sCtx2)
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

		sCtx := tantivy_go.NewSearchContextBuilder().
			SetQuery("Idées fête").
			SetDocsLimit(100).
			SetWithHighlights(true).
			AddFieldDefaultWeight(NameTitle).
			Build()
		result, err := tc.Search(sCtx)
		require.NoError(t, err)

		size, err := result.GetSize()
		defer result.Free()
		require.Equal(t, 1, int(size))

		sCtx2 := tantivy_go.NewSearchContextBuilder().
			SetQuery("idees fete").
			SetDocsLimit(100).
			SetWithHighlights(true).
			AddFieldDefaultWeight(NameTitle).
			Build()
		result2, err := tc.Search(sCtx2)
		require.NoError(t, err)

		size2, err := result2.GetSize()
		defer result2.Free()
		require.Equal(t, 1, int(size2))

		sCtx3 := tantivy_go.NewSearchContextBuilder().
			SetQuery("straße").
			SetDocsLimit(100).
			SetWithHighlights(true).
			AddFieldDefaultWeight(NameBody).
			Build()
		result3, err := tc.Search(sCtx3)
		require.NoError(t, err)

		size3, err := result3.GetSize()
		defer result3.Free()
		require.Equal(t, 1, int(size3))

		sCtx4 := tantivy_go.NewSearchContextBuilder().
			SetQuery("strasse").
			SetDocsLimit(100).
			SetWithHighlights(true).
			AddFieldDefaultWeight(NameBody).
			Build()
		result4, err := tc.Search(sCtx4)
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

		sCtx := tantivy_go.NewSearchContextBuilder().
			SetQuery("1").
			SetDocsLimit(100).
			SetWithHighlights(true).
			AddFieldDefaultWeight(NameId).
			Build()
		result, err := tc.Search(sCtx)
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

		sCtx := tantivy_go.NewSearchContextBuilder().
			SetQuery("create").
			SetDocsLimit(100).
			SetWithHighlights(true).
			AddFieldDefaultWeight(NameTitle).
			Build()
		result, err := tc.Search(sCtx)
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

	t.Run("docs search - when jieba", func(t *testing.T) {
		_, tc := fx(t, limit, 1, false)

		defer tc.Free()

		doc, err := addDoc(t, "", "张华考上了北京大学；李萍进了中等技术学校；我在百货公司当售货员：我们都有光明的前途", "1", tc)
		require.NoError(t, err)

		doc2, err := addDoc(t, "张华考上了北京大学；李萍进了中等技术学校；我在百货公司当售货员：我们都有光明的前途", "", "2", tc)
		require.NoError(t, err)

		err = tc.AddAndConsumeDocuments(doc, doc2)
		require.NoError(t, err)

		docs, err := tc.NumDocs()
		require.NoError(t, err)
		require.Equal(t, uint64(2), docs)

		sCtx := tantivy_go.NewSearchContextBuilder().
			SetQuery("售货员").
			SetDocsLimit(100).
			SetWithHighlights(true).
			AddFieldDefaultWeight(NameBodyZh).
			AddFieldDefaultWeight(NameTitleZh).
			Build()
		result, err := tc.Search(sCtx)
		require.NoError(t, err)

		size, err := result.GetSize()
		defer result.Free()
		require.Equal(t, 2, int(size))
	})

	t.Run("docs search - when weights apply", func(t *testing.T) {
		schema, tc := fx(t, limit, 1, false)

		defer tc.Free()

		doc, err := addDoc(t, "an apple", "", "id1", tc)
		require.NoError(t, err)

		doc2, err := addDoc(t, "", "an apple", "id2", tc)
		require.NoError(t, err)

		err = tc.AddAndConsumeDocuments(doc, doc2)
		require.NoError(t, err)

		docs, err := tc.NumDocs()
		require.NoError(t, err)
		require.Equal(t, uint64(2), docs)

		sCtx := tantivy_go.NewSearchContextBuilder().
			SetQuery("apple").
			SetDocsLimit(100).
			SetWithHighlights(false).
			AddField(NameTitle, 1.0).
			AddField(NameBody, 1.0).
			Build()
		result, err := tc.Search(sCtx)
		require.NoError(t, err)

		size, err := result.GetSize()
		defer result.Free()
		require.Equal(t, 2, int(size))
		resDoc, err := result.Get(0)
		require.NoError(t, err)
		jsonStr, err := resDoc.ToJson(schema, NameId)
		require.NoError(t, err)
		require.JSONEq(t, `{"highlights":[],"id":"id1","score":1.9676434993743896}`, jsonStr)

		sCtx2 := tantivy_go.NewSearchContextBuilder().
			SetQuery("apple").
			SetDocsLimit(100).
			SetWithHighlights(false).
			AddField(NameTitle, 1.0).
			AddField(NameBody, 10.0).
			Build()
		result2, err := tc.Search(sCtx2)
		require.NoError(t, err)

		size2, err := result2.GetSize()
		defer result2.Free()
		require.Equal(t, 2, int(size2))
		resDoc2, err := result2.Get(0)
		require.NoError(t, err)
		jsonStr2, err := resDoc2.ToJson(schema, NameId)
		require.NoError(t, err)
		require.JSONEq(t, `{"highlights":[],"id":"id2","score":4.919108867645264}`, jsonStr2)
	})
}

func addDoc(
	t *testing.T,
	title string,
	body string,
	id string,
	tc *tantivy_go.TantivyContext,
) (*tantivy_go.Document, error) {
	doc := tantivy_go.NewDocument()

	err := doc.AddField(NameTitle, title, tc)
	require.NoError(t, err)

	err = doc.AddField(NameTitleZh, title, tc)
	require.NoError(t, err)

	err = doc.AddField(NameId, id, tc)
	require.NoError(t, err)

	err = doc.AddField(NameBody, body, tc)
	require.NoError(t, err)

	err = doc.AddField(NameBodyZh, body, tc)
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
		NameTitleZh,
		true,
		true,
		false,
		tantivy_go.IndexRecordOptionWithFreqsAndPositions,
		tantivy_go.TokenizerJieba,
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

	err = builder.AddTextField(
		NameBodyZh,
		true,
		true,
		false,
		tantivy_go.IndexRecordOptionWithFreqsAndPositions,
		tantivy_go.TokenizerJieba,
	)
	require.NoError(t, err)

	schema, err := builder.BuildSchema()
	require.NoError(t, err)

	_ = os.RemoveAll("index_dir")
	tc, err := tantivy_go.NewTantivyContextWithSchema("index_dir", schema)
	require.NoError(t, err)

	err = tc.RegisterTextAnalyzerSimple(tantivy_go.TokenizerSimple, limit, tantivy_go.English)
	require.NoError(t, err)

	err = tc.RegisterTextAnalyzerJieba(tantivy_go.TokenizerJieba, limit)
	require.NoError(t, err)

	err = tc.RegisterTextAnalyzerEdgeNgram(tantivy_go.TokenizerEdgeNgram, minGram, 4, 100)
	require.NoError(t, err)

	err = tc.RegisterTextAnalyzerRaw(tantivy_go.TokenizerRaw)
	require.NoError(t, err)
	return schema, tc
}
