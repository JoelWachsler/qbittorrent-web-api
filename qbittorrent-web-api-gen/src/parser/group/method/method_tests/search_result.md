## Get search results ##

Name: `results`

**Parameters:**

Parameter                         | Type    | Description
----------------------------------|---------|------------
`id`                              | number  | ID of the search job
`limit` _optional_                | number  | max number of results to return. 0 or negative means no limit
`offset` _optional_               | number  | result to start at. A negative number means count backwards (e.g. `-2` returns the 2 most recent results)

**Returns:**

HTTP Status Code                  | Scenario
----------------------------------|---------------------
404                               | Search job was not found
409                               | Offset is too large, or too small (e.g. absolute value of negative number is greater than # results)
200                               | All other scenarios- see JSON below

The response is a JSON object with the following fields

Field                             | Type    | Description
----------------------------------|---------|------------
`results`                         | array   | Array of `result` objects- see table below
`status`                          | string  | Current status of the search job (either `Running` or `Stopped`)
`total`                           | number  | Total number of results. If the status is `Running` this number may continue to increase

**Result object:**

Field                             | Type    | Description
----------------------------------|---------|------------
`descrLink`                       | string  | URL of the torrent's description page
`fileName`                        | string  | Name of the file
`fileSize`                        | number  | Size of the file in Bytes
`fileUrl`                         | string  | Torrent download link (usually either .torrent file or magnet link)
`nbLeechers`                      | number  | Number of leechers
`nbSeeders`                       | number  | Number of seeders
`siteUrl`                         | string  | URL of the torrent site

Example:

```JSON
{
    "results": [
        {
            "descrLink": "http://www.legittorrents.info/index.php?page=torrent-details&id=8d5f512e1acb687029b8d7cc6c5a84dce51d7a41",
            "fileName": "Ubuntu-10.04-32bit-NeTV.ova",
            "fileSize": -1,
            "fileUrl": "http://www.legittorrents.info/download.php?id=8d5f512e1acb687029b8d7cc6c5a84dce51d7a41&f=Ubuntu-10.04-32bit-NeTV.ova.torrent",
            "nbLeechers": 1,
            "nbSeeders": 0,
            "siteUrl": "http://www.legittorrents.info"
        },
        {
            "descrLink": "http://www.legittorrents.info/index.php?page=torrent-details&id=d5179f53e105dc2c2401bcfaa0c2c4936a6aa475",
            "fileName": "mangOH-Legato-17_06-Ubuntu-16_04.ova",
            "fileSize": -1,
            "fileUrl": "http://www.legittorrents.info/download.php?id=d5179f53e105dc2c2401bcfaa0c2c4936a6aa475&f=mangOH-Legato-17_06-Ubuntu-16_04.ova.torrent",
            "nbLeechers": 0,
            "nbSeeders": 59,
            "siteUrl": "http://www.legittorrents.info"
        }
    ],
    "status": "Running",
    "total": 2
}
```