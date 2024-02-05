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

## Building 🔨 <a name="building"></a>

``` bash
docker build -t rustic-poker .
```

## Running 💻 <a name="running"></a>

``` bash
docker run --name=rustic-poker -p 55100:55100 rustic-poker
```

## Examples 🧪 <a name="examples"></a>

The examples provided utilize [`grpcurl`](https://github.com/fullstorydev/grpcurl) as the gRPC client.<br />
You may alternatively build your own client or choose any API testing tool of your choice.<br />
The `awesome-grpc` repo maintains a [comprehensive list of useful tooling](https://github.com/grpc-ecosystem/awesome-grpc#tools).

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
