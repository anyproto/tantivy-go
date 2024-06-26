use tantivy::tokenizer::*;
use tantivy::tokenizer::Token;
use unicode_segmentation::UnicodeSegmentation;

#[derive(Clone)]
pub struct EdgeNgramTokenizer {
    min_gram: usize,
    max_gram: usize,
    limit: usize,
}

impl EdgeNgramTokenizer {
    pub fn new(min_gram: usize,
               max_gram: usize,
               limit: usize,
    ) -> EdgeNgramTokenizer {
        EdgeNgramTokenizer {
            min_gram,
            max_gram,
            limit,
        }
    }
}

impl Tokenizer for EdgeNgramTokenizer {
    type TokenStream<'a> = BoxTokenStream<'a>;

    fn token_stream<'a>(&'a mut self, text: &'a str) -> BoxTokenStream<'a> {
        let mut copied_graphemes = String::new();
        for (pos, grapheme) in text.grapheme_indices(true) {
            if pos == self.limit { break; }
            copied_graphemes.push_str(grapheme);
        }
        let text = copied_graphemes;
        let mut tokens = Vec::new();
        let words = text.unicode_words().collect::<Vec<&str>>();
        let mut graphemes_count = 0;
        for (position, word) in words.iter().enumerate() {
            if graphemes_count == self.limit { break; }
            let graphemes = word.graphemes(true);
            let word_len = graphemes.count();
            if word_len < self.min_gram {
                continue;
            }
            let max = std::cmp::min(self.max_gram, word_len);
            for n in self.min_gram..=max {
                let mut copied_graphemes = String::new();
                for grapheme in word.graphemes(true).take(n) {
                    copied_graphemes.push_str(grapheme);
                }
                graphemes_count += 1;
                tokens.push(Token {
                    offset_from: 0,
                    offset_to: n,
                    text: copied_graphemes,
                    position,
                    position_length: 1,
                });
            }
        }
        BoxTokenStream::new(VecTokenStream { tokens, next_index: 0 })
    }
}

struct VecTokenStream {
    tokens: Vec<Token>,
    next_index: usize,
}

impl TokenStream for VecTokenStream {
    fn advance(&mut self) -> bool {
        if self.next_index < self.tokens.len() {
            self.next_index += 1;
            true
        } else {
            false
        }
    }

    fn token(&self) -> &Token {
        &self.tokens[self.next_index - 1]
    }

    fn token_mut(&mut self) -> &mut Token {
        &mut self.tokens[self.next_index - 1]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tantivy::tokenizer::Tokenizer;
    use tantivy::tokenizer::Token;

    #[test]
    fn test_edge_ngram_tokenizer_thai() {
        let mut tokenizer = EdgeNgramTokenizer::new(1, 4, 20);
        let mut token_stream = tokenizer.token_stream("ตัวอย่ง");

        let expected_tokens = vec![
            Token { offset_from: 0, offset_to: 1, position: 0, text: "ตั".to_string(), position_length: 1 },
            Token { offset_from: 0, offset_to: 1, position: 1, text: "ว".to_string(), position_length: 1 },
            Token { offset_from: 0, offset_to: 1, position: 2, text: "อ".to_string(), position_length: 1 },
            Token { offset_from: 0, offset_to: 1, position: 3, text: "ย่".to_string(), position_length: 1 },
            Token { offset_from: 0, offset_to: 1, position: 4, text: "ง".to_string(), position_length: 1 },
        ];

        for expected_token in expected_tokens {
            assert!(token_stream.advance());
            let token = token_stream.token();
            assert_eq!(token, &expected_token);
        }

        assert!(!token_stream.advance());
    }

    #[test]
    fn test_edge_ngram_tokenizer_basic() {
        let mut tokenizer = EdgeNgramTokenizer::new(2, 5, 20);
        let mut token_stream = tokenizer.token_stream("hello my friend");

        let expected_tokens = vec![
            Token { offset_from: 0, offset_to: 2, position: 0, text: "he".to_string(), position_length: 1 },
            Token { offset_from: 0, offset_to: 3, position: 0, text: "hel".to_string(), position_length: 1 },
            Token { offset_from: 0, offset_to: 4, position: 0, text: "hell".to_string(), position_length: 1 },
            Token { offset_from: 0, offset_to: 5, position: 0, text: "hello".to_string(), position_length: 1 },
            Token { offset_from: 0, offset_to: 2, position: 1, text: "my".to_string(), position_length: 1 },
            Token { offset_from: 0, offset_to: 2, position: 2, text: "fr".to_string(), position_length: 1 },
            Token { offset_from: 0, offset_to: 3, position: 2, text: "fri".to_string(), position_length: 1 },
            Token { offset_from: 0, offset_to: 4, position: 2, text: "frie".to_string(), position_length: 1 },
            Token { offset_from: 0, offset_to: 5, position: 2, text: "frien".to_string(), position_length: 1 },
        ];

        for expected_token in expected_tokens {
            assert!(token_stream.advance());
            let token = token_stream.token();
            assert_eq!(token, &expected_token);
        }

        assert!(!token_stream.advance());
    }

    #[test]
    fn test_edge_ngram_tokenizer_with_limit() {
        let mut tokenizer = EdgeNgramTokenizer::new(2, 5, 10);
        let mut token_stream = tokenizer.token_stream("hello my friend");

        let expected_tokens = vec![
            Token { offset_from: 0, offset_to: 2, position: 0, text: "he".to_string(), position_length: 1 },
            Token { offset_from: 0, offset_to: 3, position: 0, text: "hel".to_string(), position_length: 1 },
            Token { offset_from: 0, offset_to: 4, position: 0, text: "hell".to_string(), position_length: 1 },
            Token { offset_from: 0, offset_to: 5, position: 0, text: "hello".to_string(), position_length: 1 },
            Token { offset_from: 0, offset_to: 2, position: 1, text: "my".to_string(), position_length: 1 },
        ];

        for expected_token in expected_tokens {
            assert!(token_stream.advance());
            let token = token_stream.token();
            assert_eq!(token, &expected_token);
        }

        assert!(!token_stream.advance());
    }

    #[test]
    fn test_edge_ngram_tokenizer_min_gram() {
        let mut tokenizer = EdgeNgramTokenizer::new(2, 3, 10);
        let mut token_stream = tokenizer.token_stream("hi my");

        let expected_tokens = vec![
            Token { offset_from: 0, offset_to: 2, position: 0, text: "hi".to_string(), position_length: 1 },
            Token { offset_from: 0, offset_to: 2, position: 1, text: "my".to_string(), position_length: 1 },
        ];

        for expected_token in expected_tokens {
            assert!(token_stream.advance());
            let token = token_stream.token();
            assert_eq!(token, &expected_token);
        }

        assert!(!token_stream.advance());
    }

    #[test]
    fn test_edge_ngram_tokenizer_empty_string() {
        let mut tokenizer = EdgeNgramTokenizer::new(1, 3, 10);
        let mut token_stream = tokenizer.token_stream("");

        assert!(!token_stream.advance());
    }

    #[test]
    fn test_edge_ngram_tokenizer_word_shorter_than_min_gram() {
        let mut tokenizer = EdgeNgramTokenizer::new(6, 10, 10);
        let mut token_stream = tokenizer.token_stream("hello");

        assert!(!token_stream.advance());
    }
}
