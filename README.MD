## Usage

Pass a csv input file to start the project, the results will be outputted to stdio in csv format

```
cargo run transactions.csv > accounts.csv
```

Run the tests using

```
cargo test
```

## Correctness
- It is assumed that all operations are in chronological order
- It is assumed that multiple disputes cannot happen for a single transaction
- It is assumed that a chargeback cannot occur for withdrawal

## Safety and Robustness
- Monetary values are stored in cent parts, to avoid floating point precision issues
- Monetary values are stored as unsigned integers, assumed that balance cannot go below zero
- Rust's enums are used as validation errors wrapped in Result monad

## Efficiency
- Application uses csv crate to parse file. Csv reader seems to use BufReader under the hood
- Application runs in a single thread
- To allow for concurrent functionality, synchronization mechanisms would need to be implemented to keep chronological transaction ordering. Multiple producers could push to a single queue and then consistent hashing router can be used for multiple consumers.
