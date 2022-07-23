## Get search plugins ##

Name: `plugins`

**Parameters:**

None

**Returns:**

HTTP Status Code                  | Scenario
----------------------------------|---------------------
200                               | All scenarios- see JSON below

The response is a JSON array of objects containing the following fields

Field                             | Type    | Description
----------------------------------|---------|------------
`enabled`                         | bool    | Whether the plugin is enabled
`fullName`                        | string  | Full name of the plugin
`name`                            | string  | Short name of the plugin
`supportedCategories`             | array   | List of category objects
`url`                             | string  | URL of the torrent site
`version`                         | string  | Installed version of the plugin

```JSON
[
    {
        "enabled": true,
        "fullName": "Legit Torrents",
        "name": "legittorrents",
        "supportedCategories": [{
            "id": "all",
            "name": "All categories"
        }, {
            "id": "anime",
            "name": "Anime"
        }, {
            "id": "books",
            "name": "Books"
        }, {
            "id": "games",
            "name": "Games"
        }, {
            "id": "movies",
            "name": "Movies"
        }, {
            "id": "music",
            "name": "Music"
        }, {
            "id": "tv",
            "name": "TV shows"
        }],
        "url": "http://www.legittorrents.info",
        "version": "2.3"
    }
]
```

**Category object:**

Field                      | Type    | Description
---------------------------|---------|------------
`id`                       | string  | Id
`name`                     | string  | Name
