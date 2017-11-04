extern crate pest;
#[macro_use]
extern crate pest_derive;

use std::fs::File;
use std::io::Read;
use pest::inputs::Input;
use pest::iterators::Pairs;
use pest::Parser;
use std::collections::HashMap;

#[cfg(debug_assertions)]
const _GRAMMAR: &'static str = include_str!("prometheus.pest"); // relative to this file

#[derive(Parser)]
#[grammar = "prometheus.pest"] // relative to src
struct PrometheusParser;

#[derive(Clone, Debug)]
struct HistogramPair {
    le_bound: f64,
    value: f64,
}

#[derive(Clone, Debug)]
enum PrometheusAggregation {
    Counter, 
    Gauge, 
    Histogram, 
    Untyped, 
}

struct SoftPrometheusBlock {
    name: Option<String>,
    description: Option<String>,
    labels: Option<HashMap<String, String>>,
    aggregation: Option<PrometheusAggregation>,
    timestamp: Option<u32>,
}

impl SoftPrometheusBlock {
    fn new() -> SoftPrometheusBlock {
        SoftPrometheusBlock {
            name: None,
            description: None,
            labels: None,
            aggregation: None,
            timestamp: None,
        }
    }

    fn name(&mut self, n: &str) -> () {
        self.name = Some(String::from(n));
    }

    fn description(&mut self, d: &str) -> () {
        self.description = Some(String::from(d));
    }

    fn aggregation(&mut self, aggr: PrometheusAggregation) -> () {
        self.aggregation = Some(aggr);
    }

    fn harden(self) -> PrometheusBlock {
        PrometheusBlock {
            name: self.name.unwrap(),
            description: self.description,
            labels: None,
            aggregation: self.aggregation.unwrap_or(PrometheusAggregation::Untyped),
            timestamp: self.timestamp,
        }
    }
}

// HASH will be by name, labels
// Necessary to have hash as we must look backward at the previous 

#[derive(Clone, Debug)]
struct PrometheusBlock {
    name: String,
    description: Option<String>,
    labels: Option<HashMap<String, String>>,
    aggregation: PrometheusAggregation,
    timestamp: Option<u32>,
}

#[derive(Clone, Debug)]
struct PrometheusPayload {
    blocks: Vec<PrometheusBlock>,
}



fn consume<I: Input>(pairs: Pairs<Rule, I>) -> PrometheusPayload {
    let mut type_assignments: HashMap<String, PrometheusAggregation> = HashMap::new();
    let mut blocks = Vec::new();

    let mut block: SoftPrometheusBlock = SoftPrometheusBlock::new();
    for pair in pairs {
        match pair.as_rule() {
            Rule::type_directive => {
                let mut tokens = pair.into_inner();
                if let Some(metric_name) = tokens.next() {
                    let aggr: PrometheusAggregation = match tokens.next().unwrap().as_rule() {
                        Rule::prm_type_counter => PrometheusAggregation::Counter,
                        Rule::prm_type_gauge => PrometheusAggregation::Gauge,
                        Rule::prm_type_histogram => PrometheusAggregation::Histogram,
                        Rule::prm_type_untyped => PrometheusAggregation::Untyped,
                        _ => unreachable!()
                    };
                    assert!(type_assignments.insert(String::from(metric_name.as_str()), aggr).is_none());
                }
            }
            // Rule::line => {
            //     for ipair in pair.into_inner() {
            //         match ipair.as_rule() {
            //             Rule::desc_directive => {
            //                 println!("DESCRIPTION: {}", ipair.into_span().as_str())
            //             },
            //             Rule::type_directive => {
            //                 for tpair.as_rule() {
            //                 }
            //                 println!("DESCRIPTION: {}", ipair.into_span().as_str())
            //             }
            //             _ => unreachable!()
            //         }
            //     }
            // }
            _ => { println!("{:?}", pair); unreachable!() }
        }
    }

    println!("ASSIGNMENTS: {:?}", type_assignments);
    PrometheusPayload {
        blocks: blocks, 
    }
}

fn main() {
    let mut file = File::open("prometheus.payload").unwrap();
    let mut data = String::new();

    file.read_to_string(&mut data).unwrap();

    println!("{:?}", consume(PrometheusParser::parse_str(Rule::payload, &data).unwrap_or_else(|e| panic!("{}", e))));
}
