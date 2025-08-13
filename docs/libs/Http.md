# HTTP Network Library (Http)
Functions for working with HTTP calls over a system network connection (async fetch, etc.). Requires the "http" feature flag to be enabled.

## Thread Pool
This library adds a thread pool in the background for processing HTTP requests, allowing Stof to keep running while requests are executed separately. Asyncronous fetch requests will create a new Stof process, which will wait for the thread pool to execute the request before returning a map with the response data. You can then await this response map when you need it, which significantly increases performance by enabling parallel HTTP requests.

# async Http.fetch(url: str, method: str = "get", body: str | blob = null, headers: map = null, timeout: seconds = null, query: map = null, bearer: str = null) -> Promise<map>
Make an HTTP request, using the thread pool in the background so that other Stof processes can continue running.
```rust
const resp = await Http.fetch("https://restcountries.com/v3.1/region/europe");
assert(resp.get('text').len() > 100);
```

