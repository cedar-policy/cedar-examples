# Cedar Java Hello World

This repository contains a simple hello world program written in Java, using Cedar.
It shows how to use the Cedar Java API to evaluate a simple policy. You can look at the gradle files to see how to build CedarJava in your own Java applications.

## Usage

The Java example works with the version of Cedar available on Maven, which (at the time of writing) is v3.1.2.

### Build

#### Building with the Maven release

You can switch to the cedar-java uber jar in `build.gradle` so that the example runs using both the CedarJava and
CedarJavaFFI libraries from the Maven release of cedar-java:3.1.2.

```shell
# Change the build.gradle to use the uber jar
sed -i '' "s/'com.cedarpolicy:cedar-java:3.1.2'/'com.cedarpolicy:cedar-java:3.1.2:uber'/" build.gradle
./gradlew build
```

#### Building locally

To build locally, pull the corresponding 3.2.x release of cedar-java and build both CedarJavaFFI and CedarJava in your workspace.
You will need to ensure that CedarJava is able to find the dynamic library of Cedar. To do that, you need to ensure the
environment variable `CEDAR_JAVA_FFI_LIB` gives the location of your `cedar_java_ffi` shared library. Typically this can be done by running `config.sh`:

```shell
git clone -b release/3.2.x https://github.com/cedar-policy/cedar-java.git
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
