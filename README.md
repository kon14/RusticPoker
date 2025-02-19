<div align="center">
<br>
<a href="https://github.com/kon14/RusticPoker" target="_blank">
    <h1>RusticPoker 🃏</h1>
</a>
<h3>A poker game service written in Rust 🦀</h3>
</div>

<hr />

The server utilizes gRPC and the implementation is free and open-source.

The code is written with a primary emphasis on legibility.<br />
Performance is barely ever going to be an issue for this use case.<br />
As such, I've opted in favor of contextual clarity over squeezing out CPU and memory optimizations at the expense of readability.

RusticPoker's server provides support for the [gRPC Server Reflection Protocol](https://github.com/grpc/grpc/blob/master/doc/server-reflection.md).<br />
If your client doesn't support gRPC reflection, you're going to have to provide it with [RusticPoker's `.proto` file](https://github.com/kon14/RusticPoker/blob/main/src/proto/rustic_poker.proto).

---

## Building 🔨 <a name="building"></a>

``` bash
# Standard Build
docker build -t rustic-poker .

# Enabling Development Features
docker build -t rustic-poker --build-arg BUILD_FEATURES="dbg_peer_addr_spoofing" .
```

## Running 💻 <a name="running"></a>

``` bash
docker run --name=rustic-poker -p 55100:55100 rustic-poker
```

---

## Documentation 📚 <a name="documentation"></a>

If you're interacting with the API through [`gRPCurl`](https://github.com/fullstorydev/grpcurl) or any similar API testing tool, you're most likely going to have to enable the following dev build features:

#### - `dbg_peer_addr_spoofing`

Enables client spoofing via the `peer-address` request metadata header.

Besides allowing for client impersonation, this is extremely relevant for any gRPC test clients.<br />
That is because the latter don't typically persist server connections and peer address ports usually change on every single connection.<br />
As such, any test code or manual API interaction incapable of relying on a persistent connection would register as a separate client on every single request!

Example Usage: `grpcurl -H 'peer-address: 0.0.0.0:55200' ...`

### [RPC Usage Examples via gRPCurl](examples/gRPCurl)

The examples provided utilize [`gRPCurl`](https://github.com/fullstorydev/grpcurl) as the gRPC client.<br />
You may alternatively build your own client or choose any API testing tool of your choice.<br />
The `awesome-grpc` repo maintains a [comprehensive list of useful tooling](https://github.com/grpc-ecosystem/awesome-grpc#tools).

I'd strongly suggest consulting the RPC docs section and checking out the project's [.proto file](proto/rustic_poker.proto).<br />
In the meantime, here's a brief usage example of a stateless RPC to get your feet wet:

### RateHands <a name="examples-rate-hands"></a>

_Request:_
``` bash
grpcurl -plaintext -d \
'{"hands": ["2H 2D 2S 2C 6S", "2H 2D 2S 2C 6S", "2H 2D 2S 6H 2C", "2H 2D 2S 2C 5S", "AH AD 3S 3H 6C", "2H 2D 6H 2S 2C"]}' \
0.0.0.0:55100 rustic_poker.RusticPoker.RateHands
```

_Response:_
``` bash
{
  "winners": [
    "2H 2D 2S 2C 6S",
    "2H 2D 2S 6H 2C",
    "2H 2D 6H 2S 2C"
  ]
}
```

Our poker hand array input contains 5 stringified poker hand representations:<br />
One of them is `Two Pairs`, while the others are `Four of a Kind`.<br />

Of the latter, only 3 are actually unique poker hands:<br />
-`Four of a Kind`: `Quads(2), Kicker(6S)`<br />
-`Four of a Kind`: `Quads(2), Kicker(6H)`<br />
-`Four of a Kind`: `Quads(2), Kicker(5S)`<br />

Regarding the 2 cases of `Quads(2), Kicker(6)`, we have:<br />
-2x str-duplicated representations of: `Quads(2), Kicker(6S)`<br />
-2x card-shuffled representations of: `Quads(2), Kicker(6H)`<br />

Both the `Two Pairs` hand and the `Four of a Kind` with the lowest kicker get eliminated.<br />
The str-duplicated hands get deduplicated, whereas the card-shuffled hands get returned as is!<br />

---

## Environment Variables 📃 <a name="env-vars"></a>

_Note: Host envs won't propagate inside the container._

|  Variable   | Description                                                    | Required | Default | Example |
|:-----------:|:---------------------------------------------------------------|:--------:|:-------:|:-------:|
| `GRPC_PORT` | Specifies the port number that the gRPC server will listen on. |  False   | `55100` | `55101` |
