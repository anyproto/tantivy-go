package tantivy_go

import (
	"encoding/json"
)

// SearchContext defines the interface for searchContext
type SearchContext interface {
	// GetQuery returns the search query string.
	GetQuery() string
	// GetDocsLimit returns the document limit as a uintptr.
	GetDocsLimit() uintptr
	// WithHighlights returns true if highlights are enabled.
	WithHighlights() bool
	// GetFieldAndWeights returns slices of field names and their corresponding weights.
	GetFieldAndWeights() ([]string, []float32)
}

// searchContext is a structure that implements SearchContext.
type searchContext struct {
	query          string
	docsLimit      uintptr
	withHighlights bool
	fieldNames     map[string]float32
}

// GetQuery returns the search query string.
func (sc *searchContext) GetQuery() string {
	return sc.query
}

// GetDocsLimit returns the document limit.
func (sc *searchContext) GetDocsLimit() uintptr {
	return sc.docsLimit
}

// WithHighlights returns the highlights flag.
func (sc *searchContext) WithHighlights() bool {
	return sc.withHighlights
}

// GetFieldNames returns a map of field names and their weights.
func (sc *searchContext) GetFieldNames() map[string]float32 {
	return sc.fieldNames
}

// GetFieldWeights returns slices of field names and their corresponding weights.
func (sc *searchContext) GetFieldAndWeights() ([]string, []float32) {
	fields := make([]string, 0, len(sc.fieldNames))
	weights := make([]float32, 0, len(sc.fieldNames))

	for field, weight := range sc.fieldNames {
		fields = append(fields, field)
		weights = append(weights, weight)
	}

	return fields, weights
}

// SearchContextBuilder is a builder structure for creating searchContext.
type SearchContextBuilder struct {
	context *searchContext
}

// NewSearchContextBuilder creates a new instance of SearchContextBuilder.
func NewSearchContextBuilder() *SearchContextBuilder {
	return &SearchContextBuilder{
		context: &searchContext{
			fieldNames: make(map[string]float32),
		},
	}
}

// SetQuery sets the query for searchContext.
func (b *SearchContextBuilder) SetQuery(query string) *SearchContextBuilder {
	b.context.query = query
	return b
}

// SetQueryFromJson sets the query for searchContext.
func (b *SearchContextBuilder) SetQueryFromJson(query *FinalQuery) *SearchContextBuilder {
	queryJson, _ := json.Marshal(query)
	b.context.query = string(queryJson)
	return b
}

// SetDocsLimit sets the docsLimit for searchContext.
func (b *SearchContextBuilder) SetDocsLimit(limit uintptr) *SearchContextBuilder {
	b.context.docsLimit = limit
	return b
}

// SetWithHighlights sets the withHighlights flag for searchContext.
func (b *SearchContextBuilder) SetWithHighlights(withHighlights bool) *SearchContextBuilder {
	b.context.withHighlights = withHighlights
	return b
}

// AddField adds a field with the specified weight to searchContext.
func (b *SearchContextBuilder) AddField(field string, weight float32) *SearchContextBuilder {
	b.context.fieldNames[field] = weight
	return b
}

// AddFieldDefaultWeight adds a field with a default weight of 1.0 to searchContext.
func (b *SearchContextBuilder) AddFieldDefaultWeight(field string) *SearchContextBuilder {
	b.context.fieldNames[field] = 1.0
	return b
}

// Build returns the constructed searchContext as an interface.
func (b *SearchContextBuilder) Build() SearchContext {
	return b.context
}
