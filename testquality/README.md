# Test Tantivy Quality

This guide explains how to evaluate the search quality using the Tantivy library for different languages. Follow these steps for setup and testing.

---

## Prerequisites

- Python 3.x
- Rust environment set up
- Go installed

---

## Steps to Test Tantivy Quality

### 1. Clone the Corpus Data

Navigate to the `testquality/python` folder and clone the dataset:

```bash
git clone https://huggingface.co/datasets/miracl/miracl
```
## 2. Prepare the Corpus

1. Navigate to the `testquality/python` folder.
2. Open `extract_corpus.py` and set the desired language for testing:
   ```python
   lang = 'en'  # Example: English
   ```
3. Run the script:
   ```bash
   python extract_corpus.py
   ``` 
4. Wait until the script finishes processing the data.

## 3. Prepare the Index
1. Open `prepare_index.py` in the `testquality/python` folder.
2. Configure the paths for the relevance_file and queries_file according to the selected language.
3. Run the script:
   ```bash
   python prepare_index.py
   ```
4. Wait until the script completes the indexing process.

## 4. Configure the Rust Index creation
1. Navigate to the `testquality/rust` folder.
2. Open `src/main.rs` and ensure the following:
    - The stemmer corresponds to the selected language.
    - Other configurations are set appropriately.
3. Run the Rust application:
   ```bash
   cargo run
   ```
4. Wait for the results.

## 5. Configure and Run the Go Test

1. Open `testquality/main.go`.
2. Adjust the following settings:
   - **`k` value**: Change the number of results you want to evaluate.
   - **Language**: Set the language to match the dataset used (e.g., `"en"` for English).
   - **Search Method**: Choose between `tantivySearch` or `searchJson` to test the desired search method.
3. Run the Go application:
   ```bash
   go run main.go