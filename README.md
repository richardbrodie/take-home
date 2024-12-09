This is a very "proof of concept"y implementation of the required transaction parser. A number of shortcuts were taken to keep the development time to a reasonable length. In particular, the following design decisions are something I would generally avoid in a real application:

- error handling shouold be more sophisticated
- the csv parsing could allocate less
- the csv parsing can't be parallelised directly, but the `Transaction` processing could be. The accounts are stored in a HashMap but could be fetched from a thread-safe data structure instead, which could be done using a `Tokio::task` or something. 
- the unit tests aren't very thorough
- `Transaction::amount` doesn't need to be an `Option<f32>` it could be a naked `f32` if there was a custom deserializer that handled the missing values in some of the transaction types
