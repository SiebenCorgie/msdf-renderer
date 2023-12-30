<div align="center">

# MSdf-Renderer

[MiniSdf](https://gitlab.com/tendsinmende/minisdf) test renderer.

</div>


# Usage

Start the renderer with `cargo run --bin msdf-renderer` The renderer uses file `sdf.minisdf` as the injected SDF code. If no such file exists,
one is created.

The renderer watches the file, and recompiles it if necessary. So feel free to live-edit. Only valid code is send to the GPU.


## Contributing

You are welcome to contribute. All contributions are licensed under the MPL v2.0.

Note that the project is currently in its early stages. Actual contribution might be difficult.

## License

The whole project is licensed under MPL v2.0, all contributions will be licensed the same. Have a look at Mozilla's [FAQ](https://www.mozilla.org/en-US/MPL/2.0/FAQ/) to see if this fits your use-case.

