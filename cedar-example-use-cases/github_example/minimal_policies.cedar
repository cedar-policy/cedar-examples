permit(principal, action == Action::"pull", resource)
            when {
                principal in resource.readers
            };

permit(principal, action == Action::"push", resource)
            when {
                principal in resource.writers
            };
