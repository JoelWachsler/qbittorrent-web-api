# Log #

All Log API methods are under "log", e.g.: `/api/v2/log/methodName`.

## Get log ##

Name: `main`

**Parameters:**

Parameter       | Type    | Description
----------------|---------|------------
`normal`        | bool    | Include normal messages (default: `true`)
`info`          | bool    | Include info messages (default: `true`)
`warning`       | bool    | Include warning messages (default: `true`)
`critical`      | bool    | Include critical messages (default: `true`)
`last_known_id` | integer | Exclude messages with "message id" <= `last_known_id` (default: `-1`)

Example:

```http
/api/v2/log/main?normal=true&info=true&warning=true&critical=true&last_known_id=-1
```

**Returns:**

HTTP Status Code                  | Scenario
----------------------------------|---------------------
200                               | All scenarios- see JSON below

The response is a JSON array in which each element is an entry of the log.

Each element of the array has the following properties:

Property    | Type    | Description
------------|---------|------------
`id`        | integer | ID of the message
`message`   | string  | Text of the message
`timestamp` | integer | Milliseconds since epoch
`type`      | integer | Type of the message: Log::NORMAL: `1`, Log::INFO: `2`, Log::WARNING: `4`, Log::CRITICAL: `8`

Example:

```JSON
[
    {
        "id":0,
        "message":"qBittorrent v3.4.0 started",
        "timestamp":1507969127860,
        "type":1
    },
    {
        "id":1,
        "message":"qBittorrent is trying to listen on any interface port: 19036",
        "timestamp":1507969127869,
        "type":2
    },
    {
        "id":2,
        "message":"Peer ID: -qB3400-",
        "timestamp":1507969127870,
        "type":1
    },
    {
        "id":3,
        "message":"HTTP User-Agent is 'qBittorrent/3.4.0'",
        "timestamp":1507969127870,
        "type":1
    },
    {
        "id":4,
        "message":"DHT support [ON]",
        "timestamp":1507969127871,
        "type":2
    },
    {
        "id":5,
        "message":"Local Peer Discovery support [ON]",
        "timestamp":1507969127871,
        "type":2
    },
    {
        "id":6,
        "message":"PeX support [ON]",
        "timestamp":1507969127871,
        "type":2
    },
    {
        "id":7,
        "message":"Anonymous mode [OFF]",
        "timestamp":1507969127871,
        "type":2
    },
    {
        "id":8,
        "message":"Encryption support [ON]",
        "timestamp":1507969127871,
        "type":2
    },
    {
        "id":9,
        "message":"Embedded Tracker [OFF]",
        "timestamp":1507969127871,
        "type":2
    },
    {
        "id":10,
        "message":"UPnP / NAT-PMP support [ON]",
        "timestamp":1507969127873,
        "type":2
    },
    {
        "id":11,
        "message":"Web UI: Now listening on port 8080",
        "timestamp":1507969127883,
        "type":1
    },
    {
        "id":12,
        "message":"Options were saved successfully.",
        "timestamp":1507969128055,
        "type":1
    },
    {
        "id":13,
        "message":"qBittorrent is successfully listening on interface :: port: TCP/19036",
        "timestamp":1507969128270,
        "type":2
    },
    {
        "id":14,
        "message":"qBittorrent is successfully listening on interface 0.0.0.0 port: TCP/19036",
        "timestamp":1507969128271,
        "type":2
    },
    {
        "id":15,
        "message":"qBittorrent is successfully listening on interface 0.0.0.0 port: UDP/19036",
        "timestamp":1507969128272,
        "type":2
    }
]
```
