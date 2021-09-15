# riet

A piet interpreter written in rust.

Should™ be fully compliant with the spec.

# Features
- arbitrary size stack with `Vec` (based on available memory)
- arbitrary size integers from the excellent `num-bigint` library
- ability to read images with limited compression artefacts due to voting behaviour when larger codel sizes are used
- ability to read a wide variety of image formats due to the awesome `image` crate.
- ability to trace operation of the program in a similar way to `npiet`, powered by the awesome `log` and `simple_logger` crates.
