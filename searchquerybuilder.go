package tantivy_go

type QueryType int

const (
	BoolQuery QueryType = iota
	PhraseQuery
	PhrasePrefixQuery
	SingleTermPrefixQuery
	None
)

type QueryModifier int

const (
	Must QueryModifier = iota
	Should
	MustNot
)

type FieldQuery struct {
	FieldIndex int     `json:"field_index"`
	TextIndex  int     `json:"text_index"`
	Boost      float64 `json:"boost"`
}

type QueryElement struct {
	Query     Query         `json:"query"`
	Modifier  QueryModifier `json:"query_modifier"`
	QueryType QueryType     `json:"query_type"`
}

type BooleanQuery struct {
	Subqueries []QueryElement `json:"subqueries"`
}

type FinalQuery struct {
	Texts  []string      `json:"texts"`
	Fields []string      `json:"fields"`
	Query  *BooleanQuery `json:"query"`
}

type QueryBuilder struct {
	texts      map[string]int
	fields     map[string]int
	textList   []string
	fieldList  []string
	subqueries []QueryElement
}

func NewQueryBuilder() *QueryBuilder {
	return &QueryBuilder{
		texts:      make(map[string]int),
		textList:   []string{},
		subqueries: []QueryElement{},
	}
}

func (qb *QueryBuilder) AddText(text string) int {
	if idx, exists := qb.texts[text]; exists {
		return idx
	}
	idx := len(qb.textList)
	qb.texts[text] = idx
	qb.textList = append(qb.textList, text)
	return idx
}

func (qb *QueryBuilder) AddField(text string) int {
	if idx, exists := qb.fields[text]; exists {
		return idx
	}
	idx := len(qb.fieldList)
	qb.texts[text] = idx
	qb.fieldList = append(qb.fieldList, text)
	return idx
}

func (qb *QueryBuilder) Query(modifier QueryModifier, field string, text string, queryType QueryType, boost float64) *QueryBuilder {
	textIndex := qb.AddText(text)
	fieldIndex := qb.AddField(field)
	qb.subqueries = append(qb.subqueries, QueryElement{
		Query: &FieldQuery{
			FieldIndex: fieldIndex,
			TextIndex:  textIndex,
			Boost:      boost,
		},
		Modifier:  modifier,
		QueryType: queryType,
	})
	return qb
}

func (qb *QueryBuilder) BooleanQuery(modifier QueryModifier, subBuilder *QueryBuilder) *QueryBuilder {
	qb.subqueries = append(qb.subqueries, QueryElement{
		Query: &BooleanQuery{
			Subqueries: subBuilder.subqueries,
		},
		Modifier:  modifier,
		QueryType: BoolQuery,
	})
	return qb
}

func (qb *QueryBuilder) Build() FinalQuery {
	return FinalQuery{
		Texts:  qb.textList,
		Fields: qb.fieldList,
		Query: &BooleanQuery{
			Subqueries: qb.subqueries,
		},
	}
}

type Query interface {
	IsQuery()
}

func (fq *FieldQuery) IsQuery() {}

func (bq *BooleanQuery) IsQuery() {}
