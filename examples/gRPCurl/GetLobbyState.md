## GetLobbyState

---

_Request:_
``` bash
grpcurl -plaintext 0.0.0.0:55100 rustic_poker.RusticPoker.GetLobbyState
```

_Response:_
``` bash
{
  "id": "13921936",
  "name": "Joker in the Pack",
  "hostUser": "kon14",
  "players": [
    {
      "id": "58130208",
      "name": "kon14",
      "credits": "1337"
    },
    {
      "id": "34566823",
      "name": "nick-the-greek",
      "credits": "23571113"
    },
    {
      "id": "85671523",
      "name": "d-arby",
      "credits": "192329"
    },
    {
      "id": "91482953",
      "name": "mzlaffx",
      "credits": "42069"
    },
  ],
  "status": "MATCHMAKING",
  "matchmakingAcceptance": {
    "kon14": true,
    "nick-the-greek": false,
    "mzlaffx": false,
    "mzlaffx": true,
  }
}
```
