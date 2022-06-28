# Toml Editor

This is the toml editor which is used by the dotreplit service to update the .replit file while preserving formatting (e.g. line spacing and comments).  

To run a dev build, just run `cargo run`. For a production build, run `cargo build --release`.  

More info about this [here](https://replit.com/@util/Design-docs#goval/dotreplit_editor.md)  

Once this is running, it reads json input from stdin and returns output through stdout.  

The json it reads in is in the format of https://datatracker.ietf.org/doc/html/rfc6902 with one slight difference. The value field is a stringified json instead of the actual json value.  

Below is an example set of operations:  
(note - these examples will have spacing and formatting to make it easier to read but when testing, this should all be removed).  

```
[
  { "Op": "add", "Field": "foo", "Value": 123 },
  { "Op": "add", "Field": "bar/1", "Value": "{\"test\": 234}"}
]
```
