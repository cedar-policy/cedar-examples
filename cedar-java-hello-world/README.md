# Cedar Java Hello World

This repository contains a simple hello world program written in Java, using Cedar.
It shows how to use the Cedar Java API to evaluate a simple policy. You can look at the gradle files to see how to build CedarJava in your own Java applications.

## Build

### Building with the Maven release
The Java example works with the version of Cedar available on Maven, which (at the time of writing) is v4.3.1. We specify the
uber jar in build.gradle so that both the CedarJava and CedarJavaFFI libraries are included.

```shell
./gradlew build
```

### Building locally

To build locally, pull the main branch of cedar-java and build both CedarJavaFFI and CedarJava in your workspace.
You will need to ensure that CedarJava is able to find the dynamic library of Cedar. To do that, you need to ensure the
environment variable `CEDAR_JAVA_FFI_LIB` gives the location of your `cedar_java_ffi` shared library. Typically this can be done by running `config.sh`:

```shell
git clone https://github.com/cedar-policy/cedar-java.git
cd cedar-java/CedarJavaFFI && cargo build
cd ../CedarJava && ./gradlew build
# Change the build.gradle to reference the jar built locally in the step above
cd ../.. && sed -i '' "s/'com.cedarpolicy:cedar-java:.*/files('cedar-java\/CedarJava\/build\/libs\/CedarJava.jar')/" build.gradle
bash config.sh && ./gradlew build
```

### Run

```shell
./gradlew test
```