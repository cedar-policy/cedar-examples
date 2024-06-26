import * as cedar from '@cedar-policy/cedar-wasm/nodejs';

describe('Authorizer tests', () => {
    test('isAuthorized test without schema, should deny', () => {
        const call: cedar.AuthorizationCall = {
            principal: { type: 'User', id: 'Victor' },
            action: { type: 'Action', id: 'pwn' },
            resource: { type: 'Code', id: 'this' },
            context: {},
            slice: {
                policies: {},
                entities: [],
                templateInstantiations: [],
            },
        };
        const authResult = cedar.isAuthorized(call);
        if (authResult.type !== 'success') {
            throw new Error(`Expected success in evaluation, got ${JSON.stringify(authResult, null, 4)}`);
        }
        expect(authResult.response.decision).toBe('Deny');
    });
    
    test('isAuthorized test without schema, should allow', () => {
        const call: cedar.AuthorizationCall = {
            principal: { type: 'User', id: 'Victor' },
            action: { type: 'Action', id: 'pwn' },
            resource: { type: 'Code', id: 'this' },
            context: {},
            slice: {
                policies: 'permit(principal, action, resource);',
                entities: [],
                templateInstantiations: [],
            },
        };
        const authResult = cedar.isAuthorized(call);
        if (authResult.type !== 'success') {
            throw new Error(`Expected success in evaluation, got ${JSON.stringify(authResult, null, 4)}`);
        }
        expect(authResult.response.decision).toBe('Allow');
    });

    test('isAuthorized test with schema, should allow', () => {
        const call: cedar.AuthorizationCall = {
            principal: { type: 'App::User', id: 'Victor' },
            action: { type: 'App::Action', id: 'pwn' },
            resource: { type: 'App::Code', id: 'this' },
            context: {},
            schema: {
                json: {
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
                                shape: {type: 'Record', attributes: {}},
                                memberOfTypes: [],
                            },
                            Code: {
                                shape: {type: 'Record', attributes: {}},
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
                }
            },
            slice: {
                policies: 'permit(principal, action, resource);',
                entities: [],
                templateInstantiations: [],
            },
        };
        const authResult = cedar.isAuthorized(call);
        if (authResult.type !== 'success') {
            throw new Error(`Expected success in evaluation, got ${JSON.stringify(authResult, null, 4)}`);
        }
        expect(authResult.response.decision).toBe('Allow');
    });

    test('isAuthorized test with invalid schema, should error', () => {
        const call: cedar.AuthorizationCall = {
            principal: { type: 'App::User', id: 'Victor' },
            action: { type: 'App::Action', id: 'pwn' },
            resource: { type: 'App::Code', id: 'this' },
            context: {},
            schema: {
                json: {
                    App: {
                        entityTypes: {
                            User: {
                                shape: {type: 'Record', attributes: {}},
                                memberOfTypes: [],
                            },
                            Code: {
                                shape: {type: 'Record', attributes: {}},
                                memberOfTypes: [],
                            },
                        },
                        actions: {}
                    }
                }
            },
            slice: {
                policies: 'permit(principal, action, resource);',
                entities: [],
                templateInstantiations: [],
            },
        };
        const authResult = cedar.isAuthorized(call);
        expect(authResult.type).toBe('failure');
        expect('errors' in authResult && authResult.errors.length).toBeGreaterThan(0);
    });
});

describe('formatter tests', () => {
    test('can format a valid policy', () => {
        const policyText = `
            permit(principal,        action, 
                
                  resource);
        `;
        const formattingResult = cedar.formatPolicies(policyText, 100, 2);
        expect(formattingResult.type).toBe('success');
        expect('formatted_policy' in formattingResult && formattingResult.formatted_policy).toBe('permit (principal, action, resource);');
    });

    test('executes successfully but returns failure when passed an invalid policy', () => {
        const policyText = `
        -''--.
        _'>   '\.-'<
     _.'     _     '._
   .'   _.='   '=._   '.
   >_   / /_\ /_\ \   _<
     / (  \o/\\o/  ) \
     >._\ .-,_)-. /_.<
 jgs     /__/ \__\ 
           '---'`;
        const formattingResult = cedar.formatPolicies(policyText, 100, 2);
        expect(formattingResult.type).toBe('error');
    });
});

describe('json policy functionality', () => {
    test('can convert policy to json', () => {
        const policyToJsonResult = cedar.policyTextToJson(
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
        console.log('@@@@@', JSON.stringify(policyToJsonResult.policy, null, 4));
    });

    test('can convert json to policy', () => {
        const jsonPolicy = {
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

        const jsonToPolicyResult = cedar.policyTextFromJson(JSON.stringify(jsonPolicy));
        if (jsonToPolicyResult.type !== 'success') {
            throw new Error(`Expected success in conversion, got ${JSON.stringify(jsonToPolicyResult, null, 4)}`);
        }
        expect(jsonToPolicyResult.policyText.includes(`User::\"123\"`)).toBe(true);
        expect(jsonToPolicyResult.policyText.includes(`action in [Action::\"pwn\", Action::\"code\"]`)).toBe(true);
        expect(jsonToPolicyResult.policyText.includes(`resource in Code::\"wasm\"`)).toBe(true);
        expect(jsonToPolicyResult.policyText.includes(`\"AVP\"`)).toBe(true);
        expect(jsonToPolicyResult.policyText.includes(`isReadonly`)).toBe(true);
        expect(jsonToPolicyResult.policyText.includes(`benchpress`)).toBe(true);
    });
});

describe('policy and template parsing', () => {
    test('checkParsePolicySet can parse a single policy', () => {
        const parsePolicySetResult = cedar.checkParsePolicySet('permit(principal, action, resource);');
        if (parsePolicySetResult.type !== 'success') {
            throw new Error(`Expected success in parsing, got ${JSON.stringify(parsePolicySetResult, null, 4)}`);
        }   
        expect(parsePolicySetResult.policies).toBe(1);
        expect(parsePolicySetResult.templates).toBe(0);
    });

    test('checkParsePolicySet fails when parsing a bad policy', () => {
        const parsePolicySetResult = cedar.checkParsePolicySet('permit(principal, action, asdfadsf);');
        if (parsePolicySetResult.type !== 'error') {
            throw new Error(`Expected error in parsing, got ${JSON.stringify(parsePolicySetResult, null, 4)}`);
        }   
        expect(Array.isArray(parsePolicySetResult.errors)).toBe(true);
        expect(parsePolicySetResult.errors.length).toBeGreaterThan(0);
    });

    test('checkParseTemplate can parse a single template', () => {
        const parseTemplateResult = cedar.checkParseTemplate('permit(principal == ?principal, action, resource);');
        if (parseTemplateResult.type !== 'success') {
            throw new Error(`Expected success in parsing, got ${JSON.stringify(parseTemplateResult, null, 4)}`);
        }   
        expect(Array.isArray(parseTemplateResult.slots)).toBe(true);
        expect(parseTemplateResult.slots[0]).toBe('?principal');
    });

    test('checkParseTemplate fails when parsing a bad template', () => {
        const parseTemplateResult = cedar.checkParseTemplate('permit(principal, action, resource);');
        if (parseTemplateResult.type !== 'error') {
            throw new Error(`Expected error in parsing due to no slots, got ${JSON.stringify(parseTemplateResult, null, 4)}`);
        }   
        expect(Array.isArray(parseTemplateResult.errors)).toBe(true);
        expect(parseTemplateResult.errors.length).toBeGreaterThan(0);
    });
});

const SCHEMA: cedar.SchemaJson = {
    App: {
        commonTypes: {
            ActionContext: {
                type: 'Record',
                attributes: {
                    pwnLevel: { type: 'Long', required: false },
                    pwnee: { type: 'Entity' , name: 'User', required: false },
                    favoriteProjects: { type: 'Set', element: { type: 'Entity', name: 'Code'}}
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
                shape: {type: 'Record', attributes: {}},
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

describe('schema related parsing', () => {
    test('can parse a valid schema', () => {
        const parseSchemaResult = cedar.checkParseSchema(JSON.stringify(SCHEMA));
        if (parseSchemaResult.type !== 'success') {
            throw new Error(`Expected success in parsing schema, got ${JSON.stringify(parseSchemaResult, null, 4)}`);
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
            validationSettings: { enabled: 'on' },
            schema: {
                json: SCHEMA,
            },
            policySet: policyText
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
            validationSettings: { enabled: 'on' },
            schema: {
                json: SCHEMA,
            },
            policySet: policyText
        });
        if (validationResult.type !== 'success') {
            throw new Error(`Expected success in validation, got ${JSON.stringify(validationResult, null, 4)}`);
        }
        expect(validationResult.validationErrors.length).toBeGreaterThan(0);
        expect(validationResult.validationWarnings.length).toBe(0);
        expect(validationResult.otherWarnings.length).toBe(0);
    });
});