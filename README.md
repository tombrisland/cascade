# Soon
* Metrics on queues
* Move to channels in async-rs
* Enable multithreaded scheduling of individual components
* Split into crates
* Test framework utilising builder (ser + de component items? add a trait bound to component)
* Decide on responsibilities between controller graph etc
* Basic UI

## Way out

* Multiple content references
* Different content types - InMemory, Disk, Http resource?
* Metrics for each processor
* Retry for processors
* Atomic operation guarantee - safe writes + nothing lost during rollback
* Transactions for each