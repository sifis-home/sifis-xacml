use std::error::Error;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use manifest::{AppLabel, Hazard};

fn read_app_label_from_file<P: AsRef<Path>>(path: P) -> Result<AppLabel, Box<dyn Error>> {

    // Open the file in read-only mode with buffer.
    let file = File::open(path)?;
    let reader = BufReader::new(file);

    // Read the JSON contents of the file as an instance of `AppLabel`.
    let app_label = serde_json::from_reader(reader)?;

    Ok(app_label)
}

fn create_request(app_name: &str, device_type: &str, action: &str, hazards: Vec<&Hazard>) -> String {
    let start_request : String = r#"
    <Request xmlns="urn:oasis:names:tc:xacml:3.0:core:schema:wd-17" CombinedDecision="false" ReturnPolicyIdList="false" >"#.to_owned();

    let subject : String = r#"
	<Attributes Category="urn:oasis:names:tc:xacml:1.0:subject-category:access-subject">
	  <Attribute IncludeInResult="false" AttributeId="urn:oasis:names:tc:xacml:1.0:subject:subject-id">
	    <AttributeValue DataType="http://www.w3.org/2001/XMLSchema#string">manifest</AttributeValue>
	  </Attribute>
    </Attributes>"#.to_owned();

    let resource : String = r#"
    <Attributes Category="urn:oasis:names:tc:xacml:3.0:attribute-category:resource">
      <Attribute AttributeId="urn:oasis:names:tc:xacml:1.0:resource:resource-id" IncludeInResult="false">
        <AttributeValue DataType="http://www.w3.org/2001/XMLSchema#string">"#.to_owned() + app_name + r#"</AttributeValue>
      </Attribute>
      <Attribute AttributeId="urn:oasis:names:tc:xacml:1.0:resource:device-type" IncludeInResult="false">
        <AttributeValue DataType="http://www.w3.org/2001/XMLSchema#string">"# + device_type + r#"</AttributeValue>
      </Attribute>
      <Attribute AttributeId="urn:oasis:names:tc:xacml:1.0:resource:action" IncludeInResult="false">
        <AttributeValue DataType="http://www.w3.org/2001/XMLSchema#string">"# + action + r#"</AttributeValue>
      </Attribute>
    </Attributes>"#;

    let action : String = r#"
    <Attributes Category="urn:oasis:names:tc:xacml:3.0:attribute-category:action">
      <Attribute AttributeId="urn:oasis:names:tc:xacml:1.0:action:action-id" IncludeInResult="false">
        <AttributeValue DataType="http://www.w3.org/2001/XMLSchema#string">install</AttributeValue>
      </Attribute>
    </Attributes>"#.to_owned();

    // Let's start by adding just the hazards
    // TODO: manage the risk score
    let mut environment: String = r#"
    <Attributes Category="urn:oasis:names:tc:xacml:3.0:attribute-category:environment">
      <Attribute AttributeId="urn:oasis:names:tc:xacml:1.0:environment:hazards" IncludeInResult="false">"#.to_owned();

    for hazard in hazards {
        let hazard_attribute : String = r#"
        <AttributeValue DataType="http://www.w3.org/2001/XMLSchema#string">"#.to_owned() + hazard.name.as_str() + r#"</AttributeValue>"#;
        environment = format!("{}{}", environment, hazard_attribute);
    }

    let end_environment : String = r#"
      </Attribute>
    </Attributes>"#.to_owned();
    environment = format!("{}{}", environment, end_environment);

    let end_request : String = r#"
    </Request>"#.to_owned();

    let request = format!("{}{}{}{}{}{}",
                           start_request, subject, resource, action, environment, end_request);

    return request;
}

fn main() {
    let app_label = read_app_label_from_file("data/app_label.json").unwrap();

    println!("Extracting XACML requests from app: \"{}\"...",app_label.app_name);

    let mut requests: Vec<String> =  Vec::new();

    // from each API label, we create an XACML request
    for api_label in app_label.api_labels {
        let app_name : &str = app_label.app_name.as_str();
        let device_type : &str = api_label.behavior_label[0].device_type.as_str();
        let action : &str = api_label.behavior_label[0].action.as_str();

        // create an array containing all the hazards of this API label
        let num_hazards = api_label.security_label.safety.capacity() +
            api_label.security_label.privacy.capacity() +
            api_label.security_label.financial.capacity();

        let mut hazards = Vec::with_capacity(num_hazards);

        for safety_hazard in api_label.security_label.safety.iter() {
            hazards.push(safety_hazard);
        }

        for privacy_hazard in api_label.security_label.privacy.iter() {
            hazards.push(privacy_hazard);
        }

        for financial_hazard in api_label.security_label.financial.iter() {
            // get the hazard name and optional risk score
            hazards.push(financial_hazard)
        }

        // create an xacml request for the current API label
        // and add it to the array of the requests
        requests.push(create_request(app_name, device_type, action, hazards));
    }

    // print each extracted request as a string
    for value in requests.iter() {
        println!("{}", value);
    }
}