# Cedar WASM examples

This repo runs tests against wasm code and exemplifies how to use many of the features.

## Running the tests

Simply install the packages with `npm i` and then run the tests with `npm t`:

```shell
cd cedar-wasm-example
npm i
npm t
```

## Testing an unreleased cedar version

First, have the [cedar](https://github.com/cedar-policy/cedar) repository checked out.

Make your changes, then build cedar wasm by using the `cedar-wasm/build-wasm.sh` script in that repository.

Then, change this package's `package.json` to point to your local built wasm package. The entry in `package.json` should look something like:

```json
"dependencies": {
    "@cedar-policy/cedar-wasm": "file:../../cedar/cedar-wasm/pkg/"
  },
```

Then run `npm i` to let npm know that the copy of `cedar-wasm` in your `node_modules` should be symlinked to your local `cedar-wasm`.

Once you do that, `npm t` to run the tests.
