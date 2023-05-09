# Cedar Java Hello World

This repository contains a simple hello world program written in Java, using Cedar. 
It shows how to use of the Cedar Java API to evaluate a simple policy. You can look at the gradle files to see how to build CedarJava in your own Java applications.


## Usage

To build, you'll need CedarJava and CedarJavaFFI. You will need to ensure that CedarJava is able to find the dynamic library of Cedar.
### Build
- checkout [cedar-policy/cedar-java](https://github.com/cedar-policy/cedar-java) to `cedar-java`

```shell
cd cedar-java/CedarJavaFFI && cargo build
cd ../../cedar-java-hello-world && ./gradlew build
```

### Run
```shell
./gradlew run
```

## Security

See [CONTRIBUTING](CONTRIBUTING.md#security-issue-notifications) for more information.

## License

This project is licensed under the Apache-2.0 License.

