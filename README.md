
# transaction_processor

## Notes

### Basics

The application builds and can be executed via the following command structure:

```bash
cargo run -- [path to csv]
```

The output is csv data sent to stdout, which can be directed into a file like so:

```bash
[cmd] > [output csv]
```

### Completeness

All transaction types are handled: deposit, withdrawal, dispute, resolve, chargeback.

Referential transactions (dispute, resolve, chargeback) are ignored if they reference a non-existant transaction. Dispute and resolution operations are idempotent, so if a transaction is already disputed then any further disputes are no-ops which return no errors.

### Correctness

I have verified to the best of my ability in a reasonable timeframe for this assignment that this program handles all cases described in the spec correctly using a combination of unit, end2end and manual tests.

All testdata used to end2end tests are included under the `testdata` folder.

### Safety and Robustness

I have not used any `unsafe Rust` in this program. I have handled all instances of Result and Option appropriately and any remaining instances of `unwrap()` have been annotated with an `// Unsafe safety` comment to aid maintainence.

In an ideal world, I would have created a custom `Error` type which encaspulates all possible errors that can occur during execution. Due to constraints, I have had to settle for using the following signature in most methods that can fail (including main).

```rust
Box<dyn std::error::Error>
```

I have elected to bubble up most errors to stdout via `main()`'s return-value so that users get the most amount of information possible about any issues that arise. I recognise that this may lead to confusing errors that mean nothing to users unfamiliar with the inner workings, but this is an alright compromise considering constraints in my opinion.

I have assumed that all dependencies handle potentially dangerous operations, such as filesystem access, properly and safely.

### Efficiency

With system resources in mind, I do not store referential transactions but instead to either mark whether a transaction is disputed using a `bool` or delete a transaction that has been charged-back. If referential transactions did not exist, I would have simply maintained a running total for each client account.

I do not deserialize the entire input .csv at once but instead opted to parse one record at a time to save memory.

Given more time, I would work to make the reader an asynchronous channel, potentially mpsc.

### Maintainability

I have explicitly designed the program to be both maintainable and extensible. I have seperated out each logical portion of the code into modules, which are imported by the main program. Almost all operations are handled by descriptively-named objects which encapsulate the data and behaviour needed to handle a specific task.

An example of this design is the `Transaction` enum, which leverages Rust's type system to result in clean and extensible code. `Transaction` leverages the `TryFrom` trait to encapulate away the complexity of constructing it.
