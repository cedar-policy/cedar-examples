import * as cedar from '@cedar-policy/cedar-wasm/nodejs';

describe('authorizer tests', () => {
    test('isAuthorized test without schema, should deny', () => {
        const call: cedar.AuthorizationCall = {
            principal: { type: 'User', id: 'Victor' },
            action: { type: 'Action', id: 'pwn' },
            resource: { type: 'Code', id: 'this' },
            context: {},
            policies: {
                staticPolicies: {},
                templates: {},
                templateLinks: [],
            },
            entities: [],
        };
        const authResult = cedar.isAuthorized(call);
        if (authResult.type !== 'success') {
            throw new Error(`Expected success in evaluation, got ${JSON.stringify(authResult, null, 4)}`);
        }
        expect(authResult.response.decision).toBe('deny');
    });

    test('isAuthorized test without schema, should allow', () => {
        const call: cedar.AuthorizationCall = {
            principal: { type: 'User', id: 'Victor' },
            action: { type: 'Action', id: 'pwn' },
            resource: { type: 'Code', id: 'this' },
            context: {},
            policies: {
                staticPolicies: 'permit(principal, action, resource);',
                templates: {},
                templateLinks: [],
            },
            entities: [],
        };
        const authResult = cedar.isAuthorized(call);
        if (authResult.type !== 'success') {
            throw new Error(`Expected success in evaluation, got ${JSON.stringify(authResult, null, 4)}`);
        }
        expect(authResult.response.decision).toBe('allow');
    });

    test('isAuthorized test with schema, should allow', () => {
        const call: cedar.AuthorizationCall = {
            principal: { type: 'App::User', id: 'Victor' },
            action: { type: 'App::Action', id: 'pwn' },
            resource: { type: 'App::Code', id: 'this' },
            context: {},
            schema: {
                App: {
                    commonTypes: {
                        ActionContext: {
                            type: 'Record',
                            attributes: {
                                pwnLevel: { type: 'Long', required: false }
                            }
                        }
                    },
                    entityTypes: {
                        User: {
                            shape: { type: 'Record', attributes: {} },
                            memberOfTypes: [],
                        },
                        Code: {
                            shape: { type: 'Record', attributes: {} },
                            memberOfTypes: [],
                        },
                    },
                    actions: {
                        pwn: {
                            appliesTo: {
                                context: {
                                    type: 'ActionContext',
                                },
                                resourceTypes: [
                                    'Code'
                                ],
                                principalTypes: [
                                    'User'
                                ]
                            }
                        }
                    }
                }
            },
            policies: {
                staticPolicies: 'permit(principal, action, resource);',
                templates: {},
                templateLinks: [],
            },
            entities: [],
        };
        const authResult = cedar.isAuthorized(call);
        if (authResult.type !== 'success') {
            throw new Error(`Expected success in evaluation, got ${JSON.stringify(authResult, null, 4)}`);
        }
        expect(authResult.response.decision).toBe('allow');
    });

    test('isAuthorized test with invalid schema, should error', () => {
        const call: cedar.AuthorizationCall = {
            principal: { type: 'App::User', id: 'Victor' },
            action: { type: 'App::Action', id: 'pwn' },
            resource: { type: 'App::Code', id: 'this' },
            context: {},
            schema: {
                App: {
                    entityTypes: {
                        User: {
                            shape: { type: 'Record', attributes: {} },
                            memberOfTypes: [],
                        },
                        Code: {
                            shape: { type: 'Record', attributes: {} },
                            memberOfTypes: [],
                        },
                    },
                    actions: {}
                }
            },
            policies: {
                staticPolicies: 'permit(principal, action, resource);',
                templates: {},
                templateLinks: [],
            },
            entities: [],
        };
        const authResult = cedar.isAuthorized(call);
        expect(authResult.type).toBe('failure');
        expect('errors' in authResult && authResult.errors.length).toBeGreaterThan(0);
    });

    test('isAuthorized test with template, should deny', () => {
        const call: cedar.AuthorizationCall = {
            principal: { type: 'User', id: 'Victor' },
            action: { type: 'Action', id: 'pwn' },
            resource: { type: 'Code', id: 'this' },
            context: {},
            policies: {
                staticPolicies: {},
                templates: {
                    "template0": "permit(principal == ?principal, action, resource == Code::\"this\");"
                },
                templateLinks: [],
            },
            entities: [],
        };
        const authResult = cedar.isAuthorized(call);
        if (authResult.type !== 'success') {
            throw new Error(`Expected success in evaluation, got ${JSON.stringify(authResult, null, 4)}`);
        }
        expect(authResult.response.decision).toBe('deny');
    });

    test('isAuthorized test with template, should allow', () => {
        const call: cedar.AuthorizationCall = {
            principal: { type: 'User', id: 'Victor' },
            action: { type: 'Action', id: 'pwn' },
            resource: { type: 'Code', id: 'this' },
            context: {},
            policies: {
                staticPolicies: {},
                templates: {
                    "template0": "permit(principal == ?principal, action, resource == Code::\"this\");"
                },
                templateLinks: [{
                    templateId: "template0",
                    newId: "policy0",
                    values: { "?principal": { type: 'User', id: 'Victor' } }
                }],
            },
            entities: [],
        };
        const authResult = cedar.isAuthorized(call);
        if (authResult.type !== 'success') {
            throw new Error(`Expected success in evaluation, got ${JSON.stringify(authResult, null, 4)}`);
        }
        expect(authResult.response.decision).toBe('allow');
    });

    test('isAuthorized test with entities, should allow', () => {
        const call: cedar.AuthorizationCall = {
            principal: { type: 'User', id: 'Victor' },
            action: { type: 'Action', id: 'pwn' },
            resource: { type: 'Code', id: 'this' },
            context: {},
            policies: {
                staticPolicies: {
                    "policy0": "permit(principal, action, resource) when { principal.isAdmin };"
                },
                templates: {},
                templateLinks: [],
            },
            entities: [{
                uid: { type: 'User', id: 'Victor' },
                attrs: {
                    "isAdmin": true,
                },
                parents: [],
            }],
        };
        const authResult = cedar.isAuthorized(call);
        if (authResult.type !== 'success') {
            throw new Error(`Expected success in evaluation, got ${JSON.stringify(authResult, null, 4)}`);
        }
        expect(authResult.response.decision).toBe('allow');
    });
});

describe('formatter tests', () => {
    test('can format a valid policy', () => {
        const call: cedar.FormattingCall = {
            lineWidth: 100,
            indentWidth: 2,
            policyText: `
            permit(principal,        action, 
                
                  resource);
        `};
        const formattingResult = cedar.formatPolicies(call);
        expect(formattingResult.type).toBe('success');
        expect('formatted_policy' in formattingResult && formattingResult.formatted_policy).toBe('permit (principal, action, resource);\n');
    });

    test('executes successfully but returns failure when passed an invalid policy', () => {
        const call: cedar.FormattingCall = {
            lineWidth: 100,
            indentWidth: 2,
            policyText: `
        -''--.
        _'>   '\.-'<
     _.'     _     '._
   .'   _.='   '=._   '.
   >_   / /_\ /_\ \   _<
     / (  \o/\\o/  ) \
     >._\ .-,_)-. /_.<
 jgs     /__/ \__\ 
           '---'`};
        const formattingResult = cedar.formatPolicies(call);
        expect(formattingResult.type).toBe('failure');
    });
});

describe('json policy functionality', () => {
    test('can convert policy to json', () => {
        const policyToJsonResult = cedar.policyToJson(
            `permit(
                principal== User::"123",
                action in [Action::"pwn", Action::"code"],
                resource in Code::"wasm"
            ) when {
                (principal.team == "AVP" && !resource.isReadonly) || principal.benchpress >= 300 
            };
            `
        );
        if (policyToJsonResult.type !== 'success') {
            throw new Error(`Expected success in conversion, got ${JSON.stringify(policyToJsonResult, null, 4)}`);
        }
        console.log('@@@@@ Result of policy to JSON conversion @@@@\n', JSON.stringify(policyToJsonResult.json, null, 4));
    });

    test('can convert json to policy', () => {
        const jsonPolicy: cedar.Policy = {
            effect: 'permit',
            principal: {
                op: '==',
                entity: {
                    type: 'User',
                    id: '123'
                }
            },
            action: {
                op: 'in',
                entities: [
                    {
                        type: 'Action',
                        id: 'pwn'
                    },
                    {
                        type: 'Action',
                        id: 'code'
                    }
                ]
            },
            resource: {
                op: 'in',
                entity: {
                    type: 'Code',
                    id: 'wasm'
                }
            },
            conditions: [
                {
                    kind: 'when',
                    body: {
                        '||': {
                            left: {
                                '&&': {
                                    left: {
                                        '==': {
                                            left: {
                                                '.': {
                                                    left: {
                                                        Var: 'principal'
                                                    },
                                                    attr: 'team'
                                                }
                                            },
                                            right: {
                                                Value: 'AVP'
                                            }
                                        }
                                    },
                                    right: {
                                        '!': {
                                            arg: {
                                                '.': {
                                                    left: {
                                                        Var: 'resource'
                                                    },
                                                    attr: 'isReadonly'
                                                }
                                            }
                                        }
                                    }
                                }
                            },
                            right: {
                                '>=': {
                                    left: {
                                        '.': {
                                            left: {
                                                Var: 'principal'
                                            },
                                            attr: 'benchpress'
                                        }
                                    },
                                    right: {
                                        Value: 300
                                    }
                                }
                            }
                        }
                    }
                }
            ]
        };

        const jsonToPolicyResult = cedar.policyToText(jsonPolicy);
        if (jsonToPolicyResult.type !== 'success') {
            throw new Error(`Expected success in conversion, got ${JSON.stringify(jsonToPolicyResult, null, 4)}`);
        }
        expect(jsonToPolicyResult.text.includes(`User::\"123\"`)).toBe(true);
        expect(jsonToPolicyResult.text.includes(`action in [Action::\"pwn\", Action::\"code\"]`)).toBe(true);
        expect(jsonToPolicyResult.text.includes(`resource in Code::\"wasm\"`)).toBe(true);
        expect(jsonToPolicyResult.text.includes(`\"AVP\"`)).toBe(true);
        expect(jsonToPolicyResult.text.includes(`isReadonly`)).toBe(true);
        expect(jsonToPolicyResult.text.includes(`benchpress`)).toBe(true);
    });
});

describe('json schema functionality', () => {
    test('can convert schema to json', () => {
        const schemaToJsonResult = cedar.schemaToJson(
            `entity User = { "name": String };
             action sendMessage appliesTo {principal: User, resource: User};
            `
        );
        if (schemaToJsonResult.type !== 'success') {
            throw new Error(`Expected success in conversion, got ${JSON.stringify(schemaToJsonResult, null, 4)}`);
        }
        console.log('@@@@@ Result of schema to JSON conversion @@@@\n', JSON.stringify(schemaToJsonResult.json, null, 4));
    });

    test('can convert json to schema', () => {
        const jsonSchema: cedar.Schema = {
            "App": {
                "entityTypes": {
                    "User": {
                        "shape": {
                            "type": "Record",
                            "attributes": {
                                "name": {"type": "String"}
                            }
                        }
                    }
                },
                "actions": {
                    "sendMessage": {
                        "appliesTo": {
                            "resourceTypes": ["User"],
                            "principalTypes": ["User"]
                        }
                    }}
                }
            };

        const jsonToSchemaResult = cedar.schemaToText(jsonSchema);
        if (jsonToSchemaResult.type !== 'success') {
            throw new Error(`Expected success in conversion, got ${JSON.stringify(jsonToSchemaResult, null, 4)}`);
        }
        expect(jsonToSchemaResult.text.includes(`  entity User = {\n    name: __cedar::String\n  };`)).toBe(true);
        expect(jsonToSchemaResult.text.includes(`  action \"sendMessage\" appliesTo {\n    principal: [User],\n    resource: [User],\n    context: {}\n  };`)).toBe(true);
    });
});

describe('policy and template parsing', () => {
    test('checkParsePolicySet can parse a single policy', () => {
        const call: cedar.PolicySet = {
            staticPolicies: 'permit(principal, action, resource);',
            templates: {},
            templateLinks: [],
        };
        const parsePolicySetResult = cedar.checkParsePolicySet(call);
        if (parsePolicySetResult.type !== 'success') {
            throw new Error(`Expected success in parsing, got ${JSON.stringify(parsePolicySetResult, null, 4)}`);
        }
    });

    test('checkParsePolicySet fails when parsing a bad policy', () => {
        const call: cedar.PolicySet = {
            staticPolicies: 'permit(principal, action, asdfadsf);',
            templates: {},
            templateLinks: [],
        };
        const parsePolicySetResult = cedar.checkParsePolicySet(call);
        if (parsePolicySetResult.type !== 'failure') {
            throw new Error(`Expected error in parsing, got ${JSON.stringify(parsePolicySetResult, null, 4)}`);
        }
        expect(Array.isArray(parsePolicySetResult.errors)).toBe(true);
        expect(parsePolicySetResult.errors.length).toBe(1);
    });

    test('checkParsePolicySet can parse a single template', () => {
        const call: cedar.PolicySet = {
            staticPolicies: {},
            templates: { 'id0': 'permit(principal == ?principal, action, resource);' },
            templateLinks: [],
        };
        const parsePolicySetResult = cedar.checkParsePolicySet(call);
        if (parsePolicySetResult.type !== 'success') {
            throw new Error(`Expected success in parsing, got ${JSON.stringify(parsePolicySetResult, null, 4)}`);
        }
    });

    test('checkParsePolicySet fails when parsing a bad template', () => {
        const call: cedar.PolicySet = {
            staticPolicies: {},
            templates: { 'id0': 'permit(principal, action, resource == ?principal);' },
            templateLinks: [],
        };
        const parsePolicySetResult = cedar.checkParsePolicySet(call);
        if (parsePolicySetResult.type !== 'failure') {
            throw new Error(`Expected error in parsing due to no slots, got ${JSON.stringify(parsePolicySetResult, null, 4)}`);
        }
        expect(Array.isArray(parsePolicySetResult.errors)).toBe(true);
        expect(parsePolicySetResult.errors.length).toBe(1);
    });
});

const SCHEMA = {
    App: {
        commonTypes: {
            ActionContext: {
                type: 'Record',
                attributes: {
                    pwnLevel: { type: 'Long', required: false },
                    pwnee: { type: 'Entity', name: 'User', required: false },
                    favoriteProjects: { type: 'Set', element: { type: 'Entity', name: 'Code' } }
                }
            },
            DemographicInfo: {
                type: 'Record',
                attributes: {
                    isCool: { type: 'Boolean' },
                    favoriteColors: { type: 'Set', element: { type: 'String' } },
                    name: { type: 'String' },
                }
            }
        },
        entityTypes: {
            User: {
                shape: {
                    type: 'Record',
                    attributes: {
                        userId: { type: 'String' },
                        demographicInfo: { type: 'DemographicInfo' },
                    },
                },
                memberOfTypes: [],
            },
            Code: {
                shape: { type: 'Record', attributes: {} },
                memberOfTypes: [],
            },
        },
        actions: {
            pwn: {
                appliesTo: {
                    context: {
                        type: 'ActionContext',
                    },
                    resourceTypes: [
                        'Code'
                    ],
                    principalTypes: [
                        'User'
                    ]
                }
            }
        }
    }
};

describe('other parsing tests', () => {
    test('can parse a valid schema', () => {
        const parseSchemaResult = cedar.checkParseSchema(SCHEMA);
        if (parseSchemaResult.type !== 'success') {
            throw new Error(`Expected success in parsing schema, got ${JSON.stringify(parseSchemaResult, null, 4)}`);
        }
    });

    test('can parse a valid context', () => {
        const call: cedar.ContextParsingCall = {
            schema: SCHEMA,
            action: { "type": "App::Action", "id": "pwn" },
            context: {
                pwnLevel: 10,
                favoriteProjects: [{ type: 'App::Code', id: 'this' }]
            },
        };
        const parseContextResult = cedar.checkParseContext(call);
        if (parseContextResult.type !== 'success') {
            throw new Error(`Expected success in parsing context, got ${JSON.stringify(parseContextResult, null, 4)}`);
        }
    });

    test('can parse valid entities', () => {
        const call: cedar.EntitiesParsingCall = {
            schema: SCHEMA,
            entities: [{
                uid: { type: 'App::User', id: 'Victor' },
                attrs: {
                    "userId": "123456",
                    "demographicInfo": {
                        isCool: true,
                        favoriteColors: ["black", "yellow"],
                        name: "Victor",
                    },
                },
                parents: [],
            }]
        };
        const parseEntitiesResult = cedar.checkParseEntities(call);
        if (parseEntitiesResult.type !== 'success') {
            throw new Error(`Expected success in parsing entities, got ${JSON.stringify(parseEntitiesResult, null, 4)}`);
        }
    });
});

describe('validator tests', () => {
    test('can validate a valid policy and schema', () => {
        const policyText = `
            permit(principal, action, resource) when {
                principal.demographicInfo.favoriteColors.containsAny(["black", "white"]) &&
                principal.demographicInfo.isCool &&
                principal.userId == "xxx" &&
                context has pwnLevel && context.pwnLevel >= 99
            };
        `;
        const validationResult = cedar.validate({
            validationSettings: { mode: "strict" },
            schema: SCHEMA,
            policies: {
                staticPolicies: policyText,
                templates: {},
                templateLinks: [],
            }
        });
        if (validationResult.type !== 'success') {
            throw new Error(`Expected success in validation, got ${JSON.stringify(validationResult, null, 4)}`);
        }
        expect(validationResult.validationErrors.length).toBe(0);
        expect(validationResult.validationWarnings.length).toBe(0);
        expect(validationResult.otherWarnings.length).toBe(0);
    });

    test('correctly returns a validation error for a missing presence check on pwnLevel', () => {
        const policyText = `
            permit(principal, action, resource) when {
                principal.demographicInfo.favoriteColors.containsAny(["black", "white"]) &&
                principal.demographicInfo.isCool &&
                principal.userId == "xxx" &&
                context.pwnLevel >= 99
            };
        `;
        const validationResult = cedar.validate({
            validationSettings: { mode: "strict" },
            schema: SCHEMA,
            policies: {
                staticPolicies: policyText,
                templates: {},
                templateLinks: [],
            }
        });
        if (validationResult.type !== 'success') {
            throw new Error(`Expected success in validation, got ${JSON.stringify(validationResult, null, 4)}`);
        }
        expect(validationResult.validationErrors.length).toBeGreaterThan(0);
        expect(validationResult.validationWarnings.length).toBe(0);
        expect(validationResult.otherWarnings.length).toBe(0);
    });
});

describe('get valid request envs', () => {
    test('issue example', () => {
        const policyJson: cedar.Policy = {
            "effect": "permit",
            "principal": {
              "op": "All"
            },
            "action": {
              "op": "All"
            },
            "resource": {
              "op": "is",
              "entity_type": "NS::R2"
            },
            "conditions": [],
            "annotations": {
              "id": "E1,E2 a,a2 R2"
            }
          };
        const schemaJson: cedar.Schema = `
        namespace NS {
            entity E;
            entity R1 in [R] = {"p1": String};
            entity R;
            entity R2 in [R] = {"p1": Long};
            entity E1 in [E] = {"p1": String};
            entity E2 in [E] = {"p1": Long};
            action "as";
            action "a" in [Action::"as"] appliesTo {
              principal: [E1, E2],
              resource: [R1, R2],
              context: {"c1": Long}
            };
            action "a1" in [Action::"as"] appliesTo {
              principal: [E1],
              resource: [R1],
              context: {"c1": Long}
            };
            action "a2" in [Action::"as"] appliesTo {
              principal: [E2],
              resource: [R2],
              context: {"c1": Long}
            };
          }
        `;

        let requestEnvs = cedar.getValidRequestEnvsPolicy(policyJson, schemaJson);
        if (requestEnvs.type !== 'success') {
            throw new Error(`Expected success in get valid request envs, got ${JSON.stringify(requestEnvs, null, 4)}`);
        }
        expect(requestEnvs.principals).toStrictEqual(['NS::E1', 'NS::E2']);
        expect(requestEnvs.actions).toStrictEqual(['NS::Action::"a"', 'NS::Action::"a2"']);
        expect(requestEnvs.resources).toStrictEqual(['NS::R2']);
    });
});
