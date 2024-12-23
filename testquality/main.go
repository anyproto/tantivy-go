package main

import (
	"encoding/json"
	"fmt"
	"github.com/anyproto/tantivy-go"
	"math"
	"os"
	"strings"
)

const NameBody = "body"
const NameId = "id"
const NameTitle = "title"

type DocSample struct {
	Id string `json:"id"`
}

type Query struct {
	Query        string   `json:"query"`
	RelevantDocs []string `json:"relevant_docs"`
}

func main() {
	const indexPath = "tantivy_index"
	const jsonQueries = "queries.json"
	var searchFunc = searchJson // or searchTantivy for the default search
	const k = 100

	err := execute(indexPath, jsonQueries, searchFunc, k)
	if err != nil {
		fmt.Println("Something went wrong:", err)
	}
}

func execute(
	indexPath string,
	jsonQueries string,
	search func(qry string, index *tantivy_go.TantivyContext, schema *tantivy_go.Schema, k uintptr) ([]*DocSample, error),
	k int,
) error {
	err := tantivy_go.LibInit(true, true, "release")
	if err != nil {
		return fmt.Errorf("failed to initialize library: %w", err)
	}

	builder, err := tantivy_go.NewSchemaBuilder()
	if err != nil {
		return fmt.Errorf("failed to create schema builder: %w", err)
	}

	err = builder.AddTextField(
		NameTitle,
		true,
		true,
		false,
		tantivy_go.IndexRecordOptionWithFreqsAndPositions,
		tantivy_go.TokenizerSimple,
	)
	if err != nil {
		return fmt.Errorf("failed to add text field: %w", err)
	}

	err = builder.AddTextField(
		NameBody,
		true,
		true,
		false,
		tantivy_go.IndexRecordOptionWithFreqsAndPositions,
		tantivy_go.TokenizerSimple,
	)
	if err != nil {
		return fmt.Errorf("failed to add text field: %w", err)
	}

	err = builder.AddTextField(
		NameId,
		true,
		false,
		false,
		tantivy_go.IndexRecordOptionWithFreqsAndPositions,
		tantivy_go.TokenizerSimple,
	)
	if err != nil {
		return fmt.Errorf("failed to add text field: %w", err)
	}

	schema, err := builder.BuildSchema()
	if err != nil {
		return fmt.Errorf("failed to build schema: %w", err)
	}

	index, err := tantivy_go.NewTantivyContextWithSchema(indexPath, schema)
	if err != nil {
		return fmt.Errorf("failed to create index: %w", err)
	}
	defer index.Free()

	err = index.RegisterTextAnalyzerSimple(tantivy_go.TokenizerSimple, 40, tantivy_go.English)
	if err != nil {
		return fmt.Errorf("failed to register text analyzer: %w", err)
	}

	jsonQueriesFile, err := os.Open(jsonQueries)
	if err != nil {
		return fmt.Errorf("failed to open file: %w", err)
	}
	defer jsonQueriesFile.Close()

	var queries []Query
	decoder := json.NewDecoder(jsonQueriesFile)
	if err := decoder.Decode(&queries); err != nil {
		return fmt.Errorf("failed to decode JSON: %w", err)
	}

	totalPrecision := 0.0
	totalRecall := 0.0
	totalNDCG := 0.0
	totalDCG := 0.0

	for _, query := range queries {
		samples, err := search(query.Query, index, schema, uintptr(k))
		if err != nil {
			return fmt.Errorf("failed search: %w", err)
		}
		ids := extractIds(samples)

		precision := CalculatePrecisionAtK(ids, query.RelevantDocs, k)
		recall := CalculateRecallAtK(ids, query.RelevantDocs, k)
		dcg := CalculateDCG(ids, query.RelevantDocs, k)
		ndcg := CalculateNDCGAtK(ids, query.RelevantDocs, k)

		fmt.Printf("Query: %s\n", query.Query)
		fmt.Printf("Precision@%d: %.2f\n", k, precision)
		fmt.Printf("Recall@%d: %.2f\n", k, recall)
		fmt.Printf("DCG@%d: %.2f\n", k, dcg)
		fmt.Printf("nDCG@%d: %.2f\n", k, ndcg)

		totalPrecision += precision
		totalRecall += recall
		totalDCG += dcg
		totalNDCG += ndcg
	}

	numQueries := float64(len(queries))
	fmt.Printf("\nMean Precision@%d: %.4f\n", k, totalPrecision/numQueries)
	fmt.Printf("Mean Recall@%d: %.4f\n", k, totalRecall/numQueries)
	fmt.Printf("Mean DCG@%d: %.4f\n", k, totalDCG/numQueries)
	fmt.Printf("Mean nDCG@%d: %.4f\n", k, totalNDCG/numQueries)
	return nil
}

func extractIds(samples []*DocSample) []string {
	ids := make([]string, len(samples))
	for i, sample := range samples {
		ids[i] = sample.Id
	}
	return ids
}

func searchTantivy(
	qry string,
	index *tantivy_go.TantivyContext,
	schema *tantivy_go.Schema,
	k uintptr,
) ([]*DocSample, error) {
	sCtx := tantivy_go.NewSearchContextBuilder().
		SetQuery(escape(qry)).
		SetDocsLimit(k).
		SetWithHighlights(false).
		AddField(NameTitle, 10).
		AddFieldDefaultWeight(NameBody).
		Build()

	result, err := index.Search(sCtx)
	if err != nil {
		return nil, fmt.Errorf("failed to search index: %w", err)
	}

	return tantivy_go.GetSearchResults(result, schema, func(jsonStr string) (*DocSample, error) {
		var doc DocSample
		return &doc, json.Unmarshal([]byte(jsonStr), &doc)
	}, NameId)
}

func escape(qry string) string {
	replacer := strings.NewReplacer(":", " ", "?", " ")
	return replacer.Replace(qry)
}

func searchJson(
	qry string,
	index *tantivy_go.TantivyContext,
	schema *tantivy_go.Schema,
	k uintptr,
) ([]*DocSample, error) {
	finalQuery := tantivy_go.NewQueryBuilder().
		Query(tantivy_go.Should, NameTitle, qry, tantivy_go.PhrasePrefixQuery, 20.0).
		Query(tantivy_go.Should, NameTitle, qry, tantivy_go.PhraseQuery, 20.0).
		Query(tantivy_go.Should, NameTitle, qry, tantivy_go.EveryTermQuery, 0.75).
		Query(tantivy_go.Should, NameTitle, qry, tantivy_go.OneOfTermQuery, 0.5).
		Query(tantivy_go.Should, NameBody, qry, tantivy_go.PhrasePrefixQuery, 1.0).
		Query(tantivy_go.Should, NameBody, qry, tantivy_go.PhraseQuery, 1.0).
		Query(tantivy_go.Should, NameBody, qry, tantivy_go.EveryTermQuery, 0.5).
		Query(tantivy_go.Should, NameBody, qry, tantivy_go.OneOfTermQuery, 0.25).
		Build()

	sCtx := tantivy_go.NewSearchContextBuilder().
		SetQueryFromJson(&finalQuery).
		SetDocsLimit(k).
		SetWithHighlights(false).
		Build()

	result, err := index.SearchJson(sCtx)
	if err != nil {
		return nil, fmt.Errorf("failed to search index: %w", err)
	}

	return tantivy_go.GetSearchResults(result, schema, func(jsonStr string) (*DocSample, error) {
		var doc DocSample
		return &doc, json.Unmarshal([]byte(jsonStr), &doc)
	}, NameId)
}

// CalculatePrecisionAtK calculates precision at rank K
func CalculatePrecisionAtK(retrievedIds []string, relevantDocs []string, k int) float64 {
	if k > len(retrievedIds) {
		k = len(retrievedIds)
	}
	relevantInTopK := 0
	relevantSet := make(map[string]struct{}, len(relevantDocs))
	for _, doc := range relevantDocs {
		relevantSet[doc] = struct{}{}
	}
	for i := 0; i < k; i++ {
		if _, exists := relevantSet[retrievedIds[i]]; exists {
			relevantInTopK++
		}
	}
	return float64(relevantInTopK) / float64(k)
}

// CalculateRecallAtK calculates recall at rank K
func CalculateRecallAtK(retrievedIds []string, relevantDocs []string, k int) float64 {
	if k > len(retrievedIds) {
		k = len(retrievedIds)
	}
	relevantInTopK := 0
	relevantSet := make(map[string]struct{}, len(relevantDocs))
	for _, doc := range relevantDocs {
		relevantSet[doc] = struct{}{}
	}
	for i := 0; i < k; i++ {
		if _, exists := relevantSet[retrievedIds[i]]; exists {
			relevantInTopK++
		}
	}
	if len(relevantDocs) == 0 {
		return 0
	}
	return float64(relevantInTopK) / float64(len(relevantDocs))
}

// CalculateDCG calculates the Discounted Cumulative Gain (DCG) at rank K
func CalculateDCG(retrievedIds []string, relevantDocs []string, k int) float64 {
	if k > len(retrievedIds) {
		k = len(retrievedIds)
	}
	relevantSet := make(map[string]struct{}, len(relevantDocs))
	for _, doc := range relevantDocs {
		relevantSet[doc] = struct{}{}
	}
	dcg := 0.0
	for i := 0; i < k; i++ {
		if _, exists := relevantSet[retrievedIds[i]]; exists {
			dcg += 1.0 / math.Log2(float64(i+2)) // i+2 because rank starts at 1
		}
	}
	return dcg
}

// CalculateNDCGAtK calculates the Normalized Discounted Cumulative Gain (NDCG) at rank K
func CalculateNDCGAtK(retrievedIds []string, relevantDocs []string, k int) float64 {
	dcgK := CalculateDCG(retrievedIds, relevantDocs, k)
	idealDCGK := CalculateDCG(relevantDocs, relevantDocs, k) // Ideal DCG assumes all relevant docs are in the top-k
	if idealDCGK == 0.0 {
		return 0.0
	}
	return dcgK / idealDCGK
}
