package tantivy_go_test

import (
	"encoding/json"
	"github.com/anyproto/tantivy-go/internal"
	"os"
	"testing"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"

	"github.com/anyproto/tantivy-go"
)

const NameBody = "body"
const NameId = "id"
const NameTitle = "title"
const NameBodyZh = "bodyZh"
const NameTitleZh = "titleZh"

const limit = 40
const minGram = 2
const maxGram = 4

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

	t.Run("docs search and remove - when by part of body", func(t *testing.T) {
		_, tc := fx(t, limit, minGram, false, false)

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

		results, err := tantivy_go.GetSearchResults(result, tc, func(jsonStr string) (interface{}, error) {
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
							"Example body doing",
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

	t.Run("docs search - when ngram", func(t *testing.T) {
		_, tc := fx(t, limit, 1, false, false)

		defer tc.Free()

		doc, err := addDoc(t, "Douglas DC-9", "", "1", tc)
		require.NoError(t, err)
		doc2, err := addDoc(t, "Werfer", "", "2", tc)
		require.NoError(t, err)

		err = tc.AddAndConsumeDocuments(doc, doc2)
		require.NoError(t, err)

		sCtx := tantivy_go.NewSearchContextBuilder().
			SetQuery("dc-9").
			SetDocsLimit(100).
			SetWithHighlights(false).
			AddFieldDefaultWeight(NameTitle).
			Build()

		result, err := tc.Search(sCtx)
		require.NoError(t, err)

		size, err := result.GetSize()
		require.Equal(t, 1, int(size))

		results, err := tantivy_go.GetSearchResults(result, tc, func(jsonStr string) (interface{}, error) {
			var doc DocSample
			return doc, json.Unmarshal([]byte(jsonStr), &doc)
		}, NameId, NameTitle, NameBody)
		require.NoError(t, err)

		require.Equal(t, len(results), int(size))
		require.NoError(t, err)

		for next := range results {
			model := results[next].(DocSample)
			require.Equal(t, DocSample{
				"Douglas DC-9",
				"1",
				"",
				[]Highlight{},
			},
				model)
		}
	})

	t.Run("docs search - when ngram json", func(t *testing.T) {
		_, tc := fx(t, limit, 1, true, false)

		defer tc.Free()

		doc, err := addDoc(t, "Douglas DC-9", "", "bafyreiarbtedoyhl54gspq7weophjxspexdlh2gr7cgjpl4bbqxefhb6p4.1lx08wya7kx2k", tc)
		require.NoError(t, err)
		doc2, err := addDoc(t, "Werfer", "", "2", tc)
		require.NoError(t, err)

		err = tc.AddAndConsumeDocuments(doc, doc2)
		require.NoError(t, err)
		build := tantivy_go.NewQueryBuilder().
			Query(tantivy_go.Must, NameId, "bafyreiarbtedoyhl54gspq7weophjxspexdlh2gr7cgjpl4bbqxefhb6p4.1lx08wya7kx2k", tantivy_go.TermQuery, 1.0).
			Query(tantivy_go.Must, NameTitle, "dc-9", tantivy_go.PhraseQuery, 1.0).
			Build()

		sCtx := tantivy_go.NewSearchContextBuilder().
			SetQueryFromJson(&build).
			SetDocsLimit(100).
			SetWithHighlights(false).
			AddFieldDefaultWeight(NameTitle).
			Build()

		result, err := tc.SearchJson(sCtx)
		require.NoError(t, err)

		size, err := result.GetSize()
		require.Equal(t, 1, int(size))

		results, err := tantivy_go.GetSearchResults(result, tc, func(jsonStr string) (interface{}, error) {
			var doc DocSample
			return doc, json.Unmarshal([]byte(jsonStr), &doc)
		}, NameId, NameTitle, NameBody)
		require.NoError(t, err)

		require.Equal(t, len(results), int(size))
		require.NoError(t, err)

		for next := range results {
			model := results[next].(DocSample)
			require.Equal(t, DocSample{
				"Douglas DC-9",
				"bafyreiarbtedoyhl54gspq7weophjxspexdlh2gr7cgjpl4bbqxefhb6p4.1lx08wya7kx2k",
				"",
				[]Highlight{},
			},
				model)
		}
	})

	t.Run("docs remove - when by body token", func(t *testing.T) {
		_, tc := fx(t, limit, minGram, false, false)

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
		_, tc := fx(t, limit, minGram, false, false)

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
		_, tc := fx(t, 1, minGram, false, false)

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
		_, tc := fxWithConfig(t, defaultTokenizerConfig().apply(func(tc *tantivyConfig) {
			tc.modifyField(NameTitle, func(field *fieldConfig) {
				field.tokenizer = tantivy_go.TokenizerEdgeNgram
			})
			tc.modifyTokenizer(tantivy_go.TokenizerEdgeNgram, func(tci *tokenizerConfigItem) {
				tci.args[0] = uintptr(1)
			})
		}))

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
		_, tc := fx(t, limit, 1, false, false)

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
		_, tc := fx(t, limit, minGram, false, false)

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
	})

	t.Run("err - when add field twice", func(t *testing.T) {
		err := internal.LibInit(true, false, "debug")
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
			NameTitle,
			true,
			true,
			false,
			tantivy_go.IndexRecordOptionWithFreqsAndPositions,
			tantivy_go.TokenizerEdgeNgram,
		)
		require.Error(t, err)
	})

	t.Run("docs fix utf8 - wrong utf8 - when lenient", func(t *testing.T) {
		_, tc := fx(t, limit, minGram, false, true)

		defer tc.Free()

		invalidUtf8Hello := string([]byte{0x68, 0x65, 0x6c, 0x6c, 0x6f, 0xff})
		doc, err := addDoc(t, "some", invalidUtf8Hello, "1", tc)
		require.NoError(t, err)

		err = tc.AddAndConsumeDocuments(doc)
		require.NoError(t, err)

		docs, err := tc.NumDocs()
		require.NoError(t, err)
		require.Equal(t, uint64(1), docs)

		sCtx := tantivy_go.NewSearchContextBuilder().
			SetQuery("1").
			SetDocsLimit(100).
			SetWithHighlights(false).
			AddFieldDefaultWeight(NameId).
			Build()
		result, err := tc.Search(sCtx)
		require.NoError(t, err)

		size, err := result.GetSize()
		require.Equal(t, 1, int(size))

		results, err := tantivy_go.GetSearchResults(result, tc, func(jsonStr string) (interface{}, error) {
			var doc DocSample
			return doc, json.Unmarshal([]byte(jsonStr), &doc)
		}, NameId, NameTitle, NameBody)
		require.NoError(t, err)

		require.Equal(t, len(results), int(size))

		for next := range results {
			model := results[next].(DocSample)
			require.Equal(t, DocSample{
				"some",
				"1",
				"hello�",
				[]Highlight{},
			},
				model)
		}
	})

	t.Run("docs fix utf8 - wrong utf8 - when not lenient", func(t *testing.T) {
		_, tc := fx(t, limit, minGram, false, false)

		defer tc.Free()

		invalidUtf8Hello := string([]byte{0x68, 0x65, 0x6c, 0x6c, 0x6f, 0xff})
		doc := tantivy_go.NewDocument()
		err := doc.AddField(NameBody, tc, invalidUtf8Hello)

		require.Error(t, err, "invalid utf-8 sequence of 1 bytes from index 5")
	})

	t.Run("docs search and remove - when title and edge", func(t *testing.T) {
		_, tc := fxWithConfig(t, defaultTokenizerConfig().apply(func(tc *tantivyConfig) {
			tc.modifyField(NameTitle, func(field *fieldConfig) {
				field.tokenizer = tantivy_go.TokenizerEdgeNgram
			})
		}))

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

		results, err := tantivy_go.GetSearchResults(result, tc, func(jsonStr string) (interface{}, error) {
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
							"Crea",
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

	t.Run("docs search and remove - when title and edge2", func(t *testing.T) {
		_, tc := fxWithConfig(t, defaultTokenizerConfig().apply(func(tc *tantivyConfig) {
			tc.modifyField(NameTitle, func(field *fieldConfig) {
				field.tokenizer = tantivy_go.TokenizerSimple
			})
		}))

		defer tc.Free()

		doc1, err := addDoc(t, "Ort", "", "1", tc)
		doc12, err := addDoc(t, "Assignee", "", "1", tc)
		doc2, err := addDoc(t, "Album", "", "2", tc)
		doc3, err := addDoc(t, "Genre", "", "3", tc)
		doc4, err := addDoc(t, "Focal", "", "4", tc)
		doc5, err := addDoc(t, "Priority", "", "5", tc)
		require.NoError(t, err)

		err = tc.AddAndConsumeDocuments(doc1, doc12, doc2, doc3, doc4, doc5)
		require.NoError(t, err)

		build := tantivy_go.NewQueryBuilder().
			Query(tantivy_go.Must, NameTitle, "o", tantivy_go.PhrasePrefixQuery, 1.0).
			Build()

		sCtx := tantivy_go.NewSearchContextBuilder().
			SetQueryFromJson(&build).
			SetDocsLimit(100).
			SetWithHighlights(true).
			AddFieldDefaultWeight(NameTitle).
			Build()
		result, err := tc.SearchJson(sCtx)
		require.NoError(t, err)

		size, err := result.GetSize()
		require.Equal(t, 1, int(size))

		//results, err := tantivy_go.GetSearchResults(result, tc, func(jsonStr string) (interface{}, error) {
		//	var doc DocSample
		//	return doc, json.Unmarshal([]byte(jsonStr), &doc)
		//}, NameId, NameTitle, NameBody)
		//require.NoError(t, err)
		//
		//require.Equal(t, len(results), int(size))
		//require.NoError(t, err)
		//
		//for next := range results {
		//	model := results[next].(DocSample)
		//	require.Equal(t, DocSample{
		//		"Create Body",
		//		"1",
		//		"Example title content.",
		//		[]Highlight{
		//			{
		//				NameTitle,
		//				Fragment{
		//					[][2]int{{0, 2}, {0, 3}, {0, 4}},
		//					"Crea",
		//				},
		//			}},
		//	},
		//		model)
		//}
		//
		//docs, err := tc.NumDocs()
		//require.NoError(t, err)
		//require.Equal(t, uint64(1), docs)
		//
		//err = tc.DeleteDocuments(NameId, "1")
		//require.NoError(t, err)
		//docs, err = tc.NumDocs()
		//require.Equal(t, uint64(0), docs)
	})

	t.Run("docs search and remove - when title and ngram", func(t *testing.T) {
		_, tc := fx(t, limit, minGram, false, false)

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

		results, err := tantivy_go.GetSearchResults(result, tc, func(jsonStr string) (interface{}, error) {
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
							[][2]int{{0, 2}, {0, 3}, {0, 4}, {1, 3}, {1, 4}, {1, 5}, {2, 4}, {2, 5}, {2, 6}, {3, 5}, {3, 6}, {4, 6}},
							"Create Body",
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
		_, tc := fx(t, limit, 1, false, false)

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

	t.Run("correct search query parse", func(t *testing.T) {
		qb := tantivy_go.NewQueryBuilder()

		finalQuery := qb.
			Query(tantivy_go.Must, "body1", "some words", tantivy_go.PhraseQuery, 1.0).
			Query(tantivy_go.Should, "body2", "term", tantivy_go.PhrasePrefixQuery, 1.0).
			Query(tantivy_go.MustNot, "body3", "term", tantivy_go.TermPrefixQuery, 1.0).
			Query(tantivy_go.Must, "title1", "another term", tantivy_go.PhraseQuery, 0.1).
			Query(tantivy_go.Should, "title2", "term2", tantivy_go.PhrasePrefixQuery, 0.1).
			Query(tantivy_go.MustNot, "title3", "term2", tantivy_go.TermPrefixQuery, 0.1).
			BooleanQuery(tantivy_go.Must, qb.NestedBuilder().
				Query(tantivy_go.Should, "summary", "term3", tantivy_go.PhrasePrefixQuery, 1.0).
				BooleanQuery(tantivy_go.Should, qb.NestedBuilder().
					Query(tantivy_go.Must, "comments", "not single term", tantivy_go.PhraseQuery, 0.8).
					Query(tantivy_go.MustNot, "body1", "sample three words", tantivy_go.EveryTermQuery, 0.4),
					0.3,
				),
				1.0,
			).
			Query(tantivy_go.Must, "body2", "sample three words", tantivy_go.OneOfTermQuery, 1.0).
			Query(tantivy_go.Must, "body2", "one", tantivy_go.PhraseQuery, 1.0).
			Query(tantivy_go.Must, "body2", "one", tantivy_go.TermQuery, 1.0).
			Build()

		expected, err := os.ReadFile("./test_jsons/data.json")
		require.NoError(t, err)

		sCtx := tantivy_go.NewSearchContextBuilder().
			SetQueryFromJson(&finalQuery).
			SetDocsLimit(100).
			SetWithHighlights(false).
			Build()

		require.JSONEq(t, string(expected), sCtx.GetQuery())
	})

	t.Run("docs search query - when prefix", func(t *testing.T) {
		_, tc := fx(t, limit, 1, false, false)

		defer tc.Free()

		doc, err := addDoc(t, "", "gaszählerstand", "id1", tc)
		require.NoError(t, err)

		err = tc.AddAndConsumeDocuments(doc)
		require.NoError(t, err)

		finalQuery := tantivy_go.NewQueryBuilder().
			Query(tantivy_go.Must, NameBody, "gaszä", tantivy_go.TermPrefixQuery, 1.0).
			Build()

		sCtx := tantivy_go.NewSearchContextBuilder().
			SetQueryFromJson(&finalQuery).
			SetDocsLimit(100).
			SetWithHighlights(false).
			Build()

		result, err := tc.SearchJson(sCtx)
		require.NoError(t, err)

		size, err := result.GetSize()
		defer result.Free()
		require.Equal(t, 1, int(size))

		sCtx2 := tantivy_go.NewSearchContextBuilder().
			SetQuery("gaszä").
			SetDocsLimit(100).
			SetWithHighlights(false).
			AddFieldDefaultWeight(NameBody).
			Build()

		result2, err := tc.Search(sCtx2)
		require.NoError(t, err)

		size2, err := result2.GetSize()
		defer result2.Free()
		require.Equal(t, 0, int(size2))
	})

	t.Run("docs search - when weights apply", func(t *testing.T) {
		_, tc := fx(t, limit, 1, false, false)

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
		jsonStr, err := resDoc.ToJson(tc, NameId)
		require.NoError(t, err)
		require.JSONEq(t, `{"highlights":[],"id":"id1","score":6.886753559112549}`, jsonStr)

		sCtx2 := tantivy_go.NewSearchContextBuilder().
			SetQuery("apple").
			SetDocsLimit(100).
			SetWithHighlights(false).
			AddField(NameTitle, 1.0).
			AddField(NameBody, 100.0).
			Build()
		result2, err := tc.Search(sCtx2)
		require.NoError(t, err)

		size2, err := result2.GetSize()
		defer result2.Free()
		require.Equal(t, 2, int(size2))
		resDoc2, err := result2.Get(0)
		require.NoError(t, err)
		jsonStr2, err := resDoc2.ToJson(tc, NameId)
		require.NoError(t, err)
		require.JSONEq(t, `{"highlights":[],"id":"id2","score":49.19108963012695}`, jsonStr2)
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

	err := doc.AddFields(title, tc, NameTitle, NameBodyZh)
	require.NoError(t, err)

	err = doc.AddField(id, tc, NameId)
	require.NoError(t, err)

	err = doc.AddFields(body, tc, NameBody, NameBodyZh)
	require.NoError(t, err)

	return doc, err
}

// Deprecated: Use fxWithConfig instead.
func fx(
	t *testing.T,
	limit uintptr,
	minGram uintptr,
	isFastId bool,
	utf8Lenient bool,
) (*tantivy_go.Schema, *tantivy_go.TantivyContext) {
	err := internal.LibInit(true, utf8Lenient, "debug")
	assert.NoError(t, err)
	builder, err := tantivy_go.NewSchemaBuilder()
	require.NoError(t, err)

	err = builder.AddTextField(
		NameTitle,
		true,
		true,
		false,
		tantivy_go.IndexRecordOptionWithFreqsAndPositions,
		tantivy_go.TokenizerNgram,
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

	err = tc.RegisterTextAnalyzerNgram(tantivy_go.TokenizerNgram, minGram, 4, false)
	require.NoError(t, err)

	err = tc.RegisterTextAnalyzerRaw(tantivy_go.TokenizerRaw)
	require.NoError(t, err)
	return schema, tc
}

type tantivyConfig struct {
	utf8Lenient      bool
	rustConfig       string
	fields           []*fieldConfig
	tokenizerConfigs []*tokenizerConfigItem
}

type fieldConfig struct {
	name         string
	stored       bool
	isText       bool
	isFast       bool
	recordOption int
	tokenizer    string
}

type tokenizerConfigItem struct {
	Type string
	args []interface{}
}

func defaultTokenizerConfig() *tantivyConfig {
	return &tantivyConfig{
		utf8Lenient: false,
		rustConfig:  "debug",
		fields: []*fieldConfig{
			{NameTitle, true, true, false, tantivy_go.IndexRecordOptionWithFreqsAndPositions, tantivy_go.TokenizerNgram},
			{NameTitleZh, true, true, false, tantivy_go.IndexRecordOptionWithFreqsAndPositions, tantivy_go.TokenizerJieba},
			{NameId, true, false, false, tantivy_go.IndexRecordOptionBasic, tantivy_go.TokenizerRaw},
			{NameBody, true, true, false, tantivy_go.IndexRecordOptionWithFreqsAndPositions, tantivy_go.TokenizerSimple},
			{NameBodyZh, true, true, false, tantivy_go.IndexRecordOptionWithFreqsAndPositions, tantivy_go.TokenizerJieba},
		},
		tokenizerConfigs: []*tokenizerConfigItem{
			{tantivy_go.TokenizerSimple, []interface{}{uintptr(100), tantivy_go.English}},
			{tantivy_go.TokenizerJieba, []interface{}{uintptr(100)}},
			{tantivy_go.TokenizerEdgeNgram, []interface{}{uintptr(minGram), uintptr(maxGram), uintptr(limit)}},
			{tantivy_go.TokenizerNgram, []interface{}{uintptr(minGram), uintptr(maxGram), false}},
			{tantivy_go.TokenizerRaw, nil},
		},
	}
}

func (tc *tantivyConfig) apply(apply func(tc *tantivyConfig)) *tantivyConfig {
	apply(tc)
	return tc
}

func (tc *tantivyConfig) modifyField(name string, modifier func(field *fieldConfig)) *tantivyConfig {
	for _, field := range tc.fields {
		if field.name == name {
			modifier(field)
		}
	}
	return tc
}

func (tc *tantivyConfig) modifyTokenizer(name string, modifier func(config *tokenizerConfigItem)) *tantivyConfig {
	for _, config := range tc.tokenizerConfigs {
		if config.Type == name {
			modifier(config)
		}
	}
	return tc
}

func modifyTokenizer(tokenizers []tokenizerConfigItem) []tokenizerConfigItem {
	for i, tokenizer := range tokenizers {
		if tokenizer.Type == tantivy_go.TokenizerNgram {
			tokenizers[i] = tokenizerConfigItem{
				Type: tantivy_go.TokenizerNgram,
				args: []interface{}{uintptr(3), uintptr(5), true}, // Измененные параметры
			}
		}
	}
	return tokenizers
}

func fxWithConfig(t *testing.T, config *tantivyConfig) (*tantivy_go.Schema, *tantivy_go.TantivyContext) {
	err := internal.LibInit(true, config.utf8Lenient, config.rustConfig)
	assert.NoError(t, err)

	builder, err := tantivy_go.NewSchemaBuilder()
	require.NoError(t, err)

	for _, field := range config.fields {
		err = builder.AddTextField(
			field.name,
			field.stored,
			field.isText,
			field.isFast,
			field.recordOption,
			field.tokenizer,
		)
		require.NoError(t, err)
	}

	schema, err := builder.BuildSchema()
	require.NoError(t, err)

	_ = os.RemoveAll("index_dir")
	tc, err := tantivy_go.NewTantivyContextWithSchema("index_dir", schema)
	require.NoError(t, err)

	for _, tokenizer := range config.tokenizerConfigs {
		switch tokenizer.Type {
		case tantivy_go.TokenizerSimple:
			err = tc.RegisterTextAnalyzerSimple(tokenizer.Type, tokenizer.args[0].(uintptr), tokenizer.args[1].(string))
		case tantivy_go.TokenizerJieba:
			err = tc.RegisterTextAnalyzerJieba(tokenizer.Type, tokenizer.args[0].(uintptr))
		case tantivy_go.TokenizerEdgeNgram:
			err = tc.RegisterTextAnalyzerEdgeNgram(tokenizer.Type, tokenizer.args[0].(uintptr), tokenizer.args[1].(uintptr), tokenizer.args[2].(uintptr))
		case tantivy_go.TokenizerNgram:
			err = tc.RegisterTextAnalyzerNgram(tokenizer.Type, tokenizer.args[0].(uintptr), tokenizer.args[1].(uintptr), tokenizer.args[2].(bool))
		case tantivy_go.TokenizerRaw:
			err = tc.RegisterTextAnalyzerRaw(tokenizer.Type)
		}
		require.NoError(t, err)
	}

	return schema, tc
}
