<div align="center">
<br>
<a href="https://github.com/kon14/RusticPoker" target="_blank">
    <h1>RusticPoker 🃏</h1>
</a>
<h3>A poker game server written in Rust 🦀</h3>
</div>

<hr />

The server utilizes gRPC and the implementation is free and open-source.

## Building 🔨 <a name="building"></a>

``` bash
docker build -t rustic-poker .
```

## Running 💻 <a name="running"></a>

``` bash
docker run --name=rustic-poker -p 55100:55100 rustic-poker
```

## Environment Variables 📃 <a name="env-vars"></a>

_Note: Host envs won't propagate inside the container._

|  Variable   | Description                                                    | Required | Default | Example |
|:-----------:|:---------------------------------------------------------------|:--------:|:-------:|:-------:|
| `GRPC_PORT` | Specifies the port number that the gRPC server will listen on. |  False   | `55100` | `55101` |
