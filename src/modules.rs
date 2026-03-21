/// A learning module available in the SWE Learn TUI.
pub struct Module {
    pub name: &'static str,
    #[allow(dead_code)] // used in detail-view screens (Phase 2)
    pub description: &'static str,
}

/// Returns the full, ordered list of learning modules.
///
/// # Examples
///
/// ```
/// use swelearn::modules::all_modules;
/// assert_eq!(all_modules().len(), 6);
/// assert_eq!(all_modules()[0].name, "Cloud Architecture");
/// ```
pub fn all_modules() -> &'static [Module] {
    &[
        Module {
            name: "Cloud Architecture",
            description: "Core cloud patterns and services used in system design",
        },
        Module {
            name: "System Design Questions",
            description: "Common interview questions with architecture diagrams",
        },
        Module {
            name: "Databases",
            description: "SQL, NoSQL, indexing, replication, and consistency models",
        },
        Module {
            name: "Networking & Protocols",
            description: "HTTP, TCP/IP, DNS, CDNs, and load balancing",
        },
        Module {
            name: "Data Structures & Algorithms",
            description: "Complexity reference and key interview patterns",
        },
        Module {
            name: "DevOps & CI/CD",
            description: "Containers, Kubernetes, observability, and SRE practices",
        },
    ]
}
