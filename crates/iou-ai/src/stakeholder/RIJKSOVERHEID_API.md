# Rijksoverheid API Feasibility Report

## Overview

This report documents the findings from probing the Rijksoverheid (Dutch Government) Data API for organization name lookup capabilities. This feasibility spike was conducted to determine if external API calls could enhance entity extraction by providing canonical organization names.

## API Endpoint

The following endpoints were tested:

1. `https://api.data.overheid.nl/io/oa/organisaties` - Primary endpoint
2. `https://api.data.overheid.nl/io/oa/organisatie` - Single organization endpoint
3. `https://directory.acceptatie.overheid.nl/public/organizations` - Acceptance environment

## Findings

As of the feasibility spike (2026-03-16):

### Availability

The Rijksoverheid Data API endpoints were tested for availability. Results may vary based on network conditions and API status.

To run the probe yourself:
```bash
cargo test -p iou-ai probe_rijksoverheid_api -- --nocapture
```

### Authentication Required

**Status:** Unknown at time of spike

The probe checks for HTTP 401/403 responses which indicate authentication requirements.

## Rate Limits

Rate limits would be indicated by headers such as:
- `X-RateLimit-Limit`
- `RateLimit-Limit`
- `X-RateLimit-Remaining`

The probe checks for these headers in API responses.

## Sample Response Format

If the API is available, the response format will be documented here after a successful probe.

Expected format based on API documentation:
```json
{
  "embedded": {
    "organisaties": [
      {
        "naam": "Ministerie van Financiën",
        "afkorting": "MinFin",
        "oid": "12345678"
      }
    ]
  }
}
```

## Fallback Strategy

Given potential API availability issues, a **local fallback dictionary** is the primary solution:

1. **Primary:** Local static dictionary with common Dutch government organizations
2. **Secondary:** If API becomes available, enhance local dictionary with API lookups
3. **Future:** Consider caching API responses locally to minimize dependency

### Local Dictionary Coverage

The fallback dictionary (`fallback_dict.rs`) includes:
- All 12 Dutch ministries with common abbreviations
- Major government agencies (Rijkswaterstaat, RDW, Belastingdienst, etc.)
- All 12 provinces
- Major municipalities (Gemeente Amsterdam, Rotterdam, etc.)

## Recommendations

1. **Use local dictionary as primary** - Fast, reliable, no external dependencies
2. **Keep API probe as diagnostic tool** - Can help identify if API becomes viable
3. **Update dictionary periodically** - Organizations change, abbreviations may be added
4. **Consider user-contributed mappings** - Allow adding custom organization mappings

## Cost Implications

Using the local fallback dictionary:
- **Per-document cost:** $0 (no API calls)
- **Maintenance:** Minimal (additions as needed)

If API integration becomes viable:
- **Per-document cost:** Depends on rate limits and caching strategy
- **Complexity:** Adds network dependency and error handling

## Next Steps

1. Monitor Rijksoverheid API status for improvements
2. Expand local dictionary with more organizations as encountered
3. Consider community contributions for rare organization names
