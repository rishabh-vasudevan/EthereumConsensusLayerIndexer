# Ethereum Consensus Layer Indexer

An indexer that indexes the attestations of validators of the last 5 epochs

## You don't have to run it locally with the help of [shuttle.rs](https://www.shuttle.rs/)

* I have deployed this code on the internet using shuttle on <https://ethereumconsensusindexer.shuttleapp.rs>
* If you want to query it you can import this [pastebin](https://pastebin.com/b5p6EjrD) link into PostMan and run the queries in Hosted App
* It takes about 6 minutes to run the indexer and the request for that is provided
* As I have already ran the indexer, you can run `get unique data` so that you can know what epoch, slots and unique validators the DB has currently and then you can make queries based on that

## If you want to run it locally

* You must have docker and rust installed
* Clone this repo
* Run the command `chmod +x script.bash`
* Then run `./script.bash` (before running the script make sure that no other PostgreSQL server is running)
* And then hopefully you will have a running server
* You can import [pastebin](https://pastebin.com/b5p6EjrD) in postman and run the queries in Local

  *__note__: I have added a docker-compose to run postgres because I am using the PostgresSQL URL with the user and password inside the code*

## To Run the Unit Tests

* You can run the `cargo test` command

## Possible Improvements for Future

* If I had multiple endpoints that I could query. Then I could run queries for all slots in all epochs concurrently (since the free endpoint only allows 20 requests per second).
* We could you cache (something like Redis), To store the network participation rate, get all unique data(which gets all the current data in the Database), etc. endpoints. This would drastically improve the response speed, As it would eliminate database calls.
