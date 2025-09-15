# HTTP Network Library (Http)
Functions for working with HTTP calls over a system network connection (async fetch, etc.). Requires the "http" feature flag to be enabled.

## Thread Pool
This library adds a thread pool in the background for processing HTTP requests, allowing Stof to keep running while requests are executed separately. Asyncronous fetch requests will create a new Stof process, which will wait for the thread pool to execute the request before returning a map with the response data. You can then await this response map when you need it, which significantly increases performance by enabling parallel HTTP requests.

# Http.blob(response: map) -> blob
Extract the body of this response as bytes.
```rust
const resp = await Http.fetch("https://restcountries.com/v3.1/region/europe");
const body = Http.blob(resp);
```

# Http.client_error(response: map) -> bool
Was the request a client error? Meaning, is the response 'status' between [400, 499]?
```rust
const resp = await Http.fetch("https://restcountries.com/v3.1/region/europe");
assert_not(Http.client_error(resp));
```

# async Http.fetch(url: str, method: str = "get", body: str | blob = null, headers: map = null, timeout: seconds = null, query: map = null, bearer: str = null) -> Promise<map>
Make an HTTP request, using the thread pool in the background so that other Stof processes can continue running.
```rust
const resp = await Http.fetch("https://restcountries.com/v3.1/region/europe");
assert(resp.get('text').len() > 100);
```

# Http.parse(response: map, context: obj = self) -> obj
Parse an HTTP response into the context object (also the return value), using the response "Content-Type" header as a Stof format (binary import). Default content type if not found in response headers is "stof". Will throw an error if the format isn't accepted by this graph, or if the body doesn't exist.
```rust
const resp = await Http.fetch("https://restcountries.com/v3.1/region/europe");
const body = new {};
try Http.parse(resp, body);
catch { /* didn't work out.. */ }
```

# Http.server_error(response: map) -> bool
Was the request a server error? Meaning, is the response 'status' between [500, 599]?
```rust
const resp = await Http.fetch("https://restcountries.com/v3.1/region/europe");
assert_not(Http.server_error(resp));
```

# Http.success(response: map) -> bool
Was the request successful? Meaning, is the response 'status' between [200, 299]?
```rust
const resp = await Http.fetch("https://restcountries.com/v3.1/region/europe");
assert(Http.success(resp));
```

# Http.text(response: map) -> str
Extract a UTF-8 text body from this response map (Equivalent to Http.blob(response) as str).
```rust
const resp = await Http.fetch("https://restcountries.com/v3.1/region/europe");
const body = Http.text(resp);
```

