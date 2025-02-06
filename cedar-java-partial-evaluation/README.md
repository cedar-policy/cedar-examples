# CedarJava Partial Evaluation Sample App
## Description
This is a sample application which uses CedarJava to showcase Cedar Partial Evaluation to:
1. Find policies and resources relevant to a given partial request (principal, action, resource = unknown)
3. Perform simple authorization for a complete request without partial evaluation

## Running the application

### Local Java Development
```
# Using gradle wrapper (recommended)
./gradlew run

# Or if gradle is installed globally 
gradle run

# To clean and rebuild
./gradlew clean build run

# To run tests
./gradlew test

```
If you get a permissions denied error while running `./gradlew run` you might have to provide permissions to gradlew using `chmod u+x ./gradlew`


### Using Docker
You can use Docker to run the sample application. To build and run the application use the following commands:
```
# Build the Docker image
docker build -t sample_app_partial_eval .

# Run the container
docker run sample_app_partial_eval
```
### Setup on EC2 Amazon Linux
Once you have setup an EC2 instance with Amazon Linux. You can install JDK21 using the following command:
```
sudo yum install java-21-amazon-corretto-devel
```
or follow the instructions [here.](https://docs.aws.amazon.com/corretto/latest/corretto-21-ug/amazon-linux-install.html)

Then you can run the application using the Gradle wrapper by executing `./gradlew run` in your terminal.

Note: The sample application has been tested with `JDK21`

## Details

### Policies, Schema, and Entities
The sample application provides basic policy-set, schema and entities:
1. Policy Set: `$PROJECTDIR/app/src/main/resources/policies.cedar` - Contains Cedar authorization policies
2. Schema: `$PROJECTDIR/app/src/main/resources/sampleapp.cedarschema` - Defines the schema for entities and their relationships
3. Entities: `$PROJECTDIR/app/src/main/resources/entities.json` - JSON representation of entities created programmatically in `cedarjavapoc_partialeval.SampleAuthorizationApp.buildEntities`. Note: CedarJava currently does not support loading entities directly from a file. This file is just a reflection of programatically created entities in the sample application. If you are modifying entities, make sure to modify `cedarjavapoc_partialeval.SampleAuthorizationApp.buildEntities`

### Sample Authorization App
`SampleAuthorizationApp` provides two public methods (excl. constructor):

1. `simpleAuthorization()` - Performs authorization evaluation for a complete request using `isAuthorized()`. 

2. `partialEvaluation()` - Performs partial authorization evaluation using `isAuthorizedPartial()` for requests where resource is unknown. Returns a list of relevant policies and entities that may determine the authorization outcome.

Users can invoke the desired method and modify parameters in `Launcher` to experiment with the functionality.

#### Partial Evaluation Details
`SampleAuthorizationApp.partialEvaluation()` currently supports partial evaluation with an unknown resource parameter. The method can be customized to handle different unknown parameters as needed. The function prints a formatted JSON containing key information about relevant policies and entities, but this represents only a subset of the data available in the full response which is returned by the method. While partial evaluation helps identify potentially relevant policies and entities, it should be used in conjunction with, not as a replacement for, full authorization checks via `isAuthorized()`.  

Example Output:  
```
====Partial Evaluation Example====
Partial Request: Principal = User::"Alice_admin", Action = Action::"Read", Resource = Unknown, Context = {}
Following policies found to be `Maybe Determining` by Partial Evaluation: 
[ {
  "PolicyID" : "policy3",
  "PolicyEffect" : "permit",
  "Relevant_Entities" : [ {
    "type" : "Document",
    "id" : "confidential_doc1"
  } ]
}, {
  "PolicyID" : "policy9",
  "PolicyEffect" : "permit",
  "Relevant_Entities" : [ {
    "type" : "DocumentGroup",
    "id" : "Public"
  } ]
}, {
  "PolicyID" : "policy0",
  "PolicyEffect" : "permit",
  "Relevant_Entities" : [ {
    "type" : "DocumentGroup",
    "id" : "Protected"
  } ]
}, {
  "PolicyID" : "policy5",
  "PolicyEffect" : "permit",
  "Relevant_Entities" : [ {
    "type" : "Document",
    "id" : "confidential_doc3"
  } ]
} ]
```