# sifis-xacml
This tool creates a set of XACML requests from an 'app label', i.e., a JSON file compliant with the [app label JSON schema](https://github.com/sifis-home/json-schemas/blob/ccf0a0f947cbf1379a1aa970b6e9bcf3a0992e37/app-label.jschema).

## Tool description
The `sifis-xacml` tool takes in input a JSON file representing an app label.

The produced XACML requests can be saved in `.xml` files at a given location, specified with the option `-o <OUTPUT_PATH>`.
They can also be printed with the option `-v`.

## View options

To view the list of `sifis-xacml` options, run:

```
cargo run -- --help
```

## Example
```
cargo run -- -a data/app_label.json -o ./my_requests/
```
```
> Creating XACML requests from app: "my_app"...

> XACML requests created successfully and saved to "./my_requests/"
```

With the option `-a <APP_LABEL_PATH>`, we specify a file containing the app label, while the option `-o <OUTPUT_PATH>` we can optionally indicate the directory we want the generated requests to be saved.

## License

Released under the [MIT License](LICENSE).

## Acknowledgements

This software has been developed in the scope of the H2020 project SIFIS-Home with GA n. 952652.