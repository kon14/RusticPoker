## GetLobbies

---

_Request:_
``` bash
grpcurl -plaintext 0.0.0.0:55100 rustic_poker.RusticPoker.GetLobbies
```

_Response:_
``` bash
{
  "lobbies": [
    {
      "id": "07639799",
      "name": "Joker in the Pack",
      "hostUser": "kon14",
      "playerCount": 4
    },
    {
      "id": "42648117",
      "name": "Stardust Crusaders",
      "hostUser": "d-arby",
      "playerCount": 2
    }
  ]
}
```
