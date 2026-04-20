// =============================================================================
// Context Layers - Domain, Semantic, and Provenance implementations
// =============================================================================

use crate::entities::*;

/// Domain context layer specialization
pub trait DomainLayer {
    fn get_primary_domain(&self) -> &Domein;
    fn add_related_domain(&mut self, domain: Domein);
    fn get_domains(&self) -> Vec<&Domein>;
}

/// Semantic context layer specialization
pub trait SemanticLayer {
    fn get_keywords(&self) -> &[String];
    fn add_keyword(&mut self, keyword: String);
    fn get_entities(&self) -> &[Entiteit];
    fn extract_entities(&self) -> Vec<Entiteit>;
}

/// Provenance context layer specialization
pub trait ProvenanceLayer {
    fn get_source_system(&self) -> &str;
    fn add_lineage_step(&mut self, step: HerkomstRecord);
    fn get_lineage(&self) -> &[HerkomstRecord];
    fn calculate_trust_score(&self) -> f64;
}
