/*
 * Copyright 2022-2023 Amazon.com, Inc. or its affiliates. All Rights Reserved.
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *      https://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

package samplelib;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertNotEquals;
import static org.junit.jupiter.api.Assertions.fail;
import com.cedarpolicy.model.exception.AuthException;
import com.cedarpolicy.model.AuthorizationResponse;

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
            sampleClass.shouldFail();
        } catch (AuthException e) {
            assertEquals("Auth Exception: Bad request: couldn't parse policy with id `p2`\nunexpected token `a`: ", e.toString());
        }
    }

}
