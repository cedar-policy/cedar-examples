package samplelib;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertNotEquals;
import static org.junit.jupiter.api.Assertions.fail;
import cedarpolicy.model.exception.AuthException;
import cedarpolicy.model.AuthorizationResult;

import org.junit.jupiter.api.Test;

/**
 *  SampleJavaClassTest.
 */
public class SampleJavaClassTest {
    @Test
    public void sampleMethodTest() {
        SampleJavaClass sampleClass = new SampleJavaClass();
	try {
        	assertEquals(true, sampleClass.sampleMethod());
	} catch (AuthException e) {
		fail("Auth Exception: " + e.toString());
	}
    }

    @Test
    public void testFailing() {
        SampleJavaClass sampleClass = new SampleJavaClass();
        try {
            AuthorizationResult r = sampleClass.shouldFail();
            assertEquals(false, r.isAllowed());
            assertNotEquals(0, r.getErrors().size());
        } catch (AuthException e) {
            fail(e.toString());
        }

    }

}
