// All users can view US security info
@id("view")
permit (
    principal,
    action == Action::"View",
    resource in Security::"US"
);

// Any user can trade stocks
@id("stocks")
permit (
    principal,
    action in Action::"Trade",
    resource is Stock
);

// Any user can trade ETFs with the following restrictions specified by forbid policies
@id("ETFs")
permit (
    principal,
    action in Action::"Trade",
    resource is ETF
);

@id("Inverse ETFs")
// Only users with high risk tolerance can trade inverse and leveraged ETFs
forbid (
    principal,
    action in Action::"Trade",
    resource is ETF
)
when { resource has inverse && resource.inverse }
unless { principal.risk_tolerance == "High" };

@id("Leveraged ETFs")
forbid (
    principal,
    action in Action::"Trade",
    resource is ETF
)
when { resource has leveraged && resource.leveraged }
unless { principal.risk_tolerance == "High" };

// Only US customers can trade crypto-based ETFs
@id("Crypto-based ETFs")
forbid (
    principal,
    action in Action::"Trade",
    resource is ETF
)
when { resource has crypto_based && resource.crypto_based }
unless { principal.location == "US" };

// Users with options level 1 can trade certain options
@id("Options level 1")
permit (
    principal,
    action in Action::"Trade",
    resource is Options
)
when { principal.options_level >= 1 }
when
{ ["Covered Call", "Covered Basket Call", "Buy Write"].contains(context.type) };

// Users with options level 2 can additionally trade certain options
@id("Options level 2")
permit (
    principal,
    action in Action::"Trade",
    resource is Options
)
when { principal.options_level >= 2 }
when { ["Long Call", "Long Put", "Covered Put"].contains(context.type) };

// Users with options level 3 can additionally trade certain options
@id("Options level 3")
permit (
    principal,
    action in Action::"Trade",
    resource is Options
)
when { principal.options_level >= 3 }
when { ["Short Put", "Synthetic", "Reversal"].contains(context.type) };

// Users with options level 4 can additionally trade certain options
@id("Options level 4")
permit (
    principal,
    action in Action::"Trade",
    resource is Options
)
when { principal.options_level >= 4 }
when
{
    ["Short Naked Call",
     "Short Straddle",
     "Short Strangle",
     "Short Synthetic"].contains
        (
            context.type
        )
};