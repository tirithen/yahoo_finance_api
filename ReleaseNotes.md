## Release 2.2.0
+ specify user agent instead of default
+ add new method `build_with_agent(self, user_agent: &str)` to allow use of custom agent
+ constructor may fail now, returning a Result

## Release 2.1.0
+ enable to retreive asset metadata
+ enable to fetch capital gains available on Mutual Funds
+ fix: support quote where firstTradeDate equals null
+ fx rate example added

## Release 2.0.1
re-export the time crate

## Release 2.0.0
Breaking change: Method `get_summary` has to be removed, since this is no longer part of the free
API interface of yahoo! finance.

## Release 1.6.1
Documentation update

## Release 1.6.0
The members `mumerator` and `denominator` of struct `Split` has been changed to from `u64` to `f64`. 
Most often, these should be small integers, but at least in some cases, the API returns these 
values as float. Fractional numerator or denominater seem to be unlikely, but not impossible,
therefore the struct was updated to accept float. Unfortunately, this is breaking change.

## Release 1.5.0
New method add `get_summary` to extract a summary of various data on a list of given quotes.
There is a new example `quote_summary` demonstrating the output.

## Release 1.4.0
Migration from chrono to time

## Release 1.3.0
`unwrap()` removed
Switch to using `thiserror` crate for error propagation
Using `Client` instance of reqwest.
Error message have possibly changed and method `build()` could fail now.
New Feature: Stop request on timeout

## Release 1.2.2
Bug fix in indexation, which in some cases caused failures when fetching the latest quote.

## Release 1.2.1
New example with blocking feature.

## Release 1.2.0
Added support for dividends and stock splits, see the new examples for splits and dividends and some code clean-up.

## Release 1.1.5
Upgrade to version 0.4.* of tokio-test

## Release 1.1.4
Mainly bug fixes and exports added for most structs. 
`search_result_opt` has been added, since sometimes not all fields are returned. These has been replaced by `Option<...>` type fields. The interface
of the `search_result` is left untouched, but returns now a default value (e.g.) empty string instead of an error.

## Release 1.1.0
New function supporting search for Quote ticker has been added, which required an additional URL path to access the Yahoo API. The previously single file project has been split up into separate files for improved maintainability. Especially, the blocking and async implementations are now
in separate files.

**Note**: Yahoo-Error type has changed. `FetchFailed` has now a string as argument instead of the status code passed over by `reqwest` to decouple the interface from `reqwest`. The former error code `InvalidStatusCode` has been renamed to `InvalidJson`, which a more proper name since this error is returned if the response could not be read as JSON. 

# Release 1.0.0
The library is working stable with and without blocking feature enabled.
