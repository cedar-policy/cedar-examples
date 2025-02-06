import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.function.Executable;

import static org.junit.jupiter.api.Assertions.*;
import cedarjavapoc_partialeval.SampleAuthorizationApp;
import com.fasterxml.jackson.databind.JsonNode;
import com.cedarpolicy.model.exception.AuthException;
import com.cedarpolicy.model.exception.InternalException;
import com.fasterxml.jackson.core.JsonProcessingException;
import com.fasterxml.jackson.databind.ObjectMapper;
import com.fasterxml.jackson.databind.node.JsonNodeFactory;
import com.fasterxml.jackson.databind.node.ObjectNode;
import java.util.Set;
import java.io.IOException;
import java.util.HashMap;
import java.util.HashSet;
import com.cedarpolicy.model.AuthorizationSuccessResponse.Decision;

public class SampleAuthorizationAppTests {

    @Test
    public void partialEvaluationTest() throws JsonProcessingException, IOException, InternalException, AuthException {
        String expectedJsonString = "[{\"PolicyID\":\"policy3\",\"PolicyEffect\":\"permit\",\"Relevant_Entities\":[{\"type\":\"Document\",\"id\":\"confidential_doc1\"}]}, "
                + "{\"PolicyID\":\"policy9\",\"PolicyEffect\":\"permit\",\"Relevant_Entities\":[{\"type\":\"DocumentGroup\",\"id\":\"Public\"}]}, "
                + "{\"PolicyID\":\"policy0\",\"PolicyEffect\":\"permit\",\"Relevant_Entities\":[{\"type\":\"DocumentGroup\",\"id\":\"Protected\"}]}, "
                + "{\"PolicyID\":\"policy5\",\"PolicyEffect\":\"permit\",\"Relevant_Entities\":[{\"type\":\"Document\",\"id\":\"confidential_doc3\"}]}]";

        String policySetFilePath = "./src/main/resources/policies.cedar";
        String schemaFilePath = "./src/main/resources/sampleapp.cedarschema";

        SampleAuthorizationApp sampleApp = new SampleAuthorizationApp(policySetFilePath, schemaFilePath);
        assertEquals(expectedJsonString,
                sampleApp.partialEvaluation("User::\"Alice_admin\"", "Action::\"Read\"", new HashMap<>()).toString());
    }

    @Test
    public void completeAuthorizationTest()
            throws JsonProcessingException, IOException, InternalException, AuthException {

        String policySetFilePath = "./src/main/resources/policies.cedar";
        String schemaFilePath = "./src/main/resources/sampleapp.cedarschema";

        SampleAuthorizationApp sampleApp = new SampleAuthorizationApp(policySetFilePath, schemaFilePath);
        assertEquals(Decision.Allow,
                sampleApp.simpleAuthorization("User::\"Alice_admin\"", "Action::\"Read\"",
                        "Document::\"protected_doc1\"", new HashMap<>()).getDecision());
    }
}
