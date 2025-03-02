use aws_sdk_ec2::{types::Filter, Client as EC2Client};
use aws_sdk_elasticloadbalancingv2::Client as ELBv2Client;
use clap::Parser;
use log::{error, info};
/// Read some lines of a file
#[derive(Debug, Parser)]
struct Cli {
    #[arg(required = true)]
    /// Target Group ARN to register instance with
    target_group_arn: String,
}

#[tokio::main]
async fn main() {
    env_logger::init();
    let args = Cli::parse();

    let shared_config = aws_config::load_from_env().await;
    let elbv2_client = ELBv2Client::new(&shared_config);
    let ec2_client = EC2Client::new(&shared_config);

    let tag_filter = Filter::builder().name("tag:Role").values("Bastion").build();

    let state_filter = Filter::builder()
        .name("instance-state-name")
        .values("running")
        .build();

    let res = ec2_client
        .describe_instances()
        .set_filters(Some(vec![tag_filter, state_filter]))
        .send()
        .await;
    if let Err(e) = res {
        error!("failed to get target health: {:?}", e);
        return;
    }

    info!("registering target group {:?}", args.target_group_arn);

    let instances = res.unwrap();
    info!("got bastion instances: {:?}", instances);

    let res = elbv2_client
        .describe_target_health()
        .target_group_arn(args.target_group_arn)
        .send()
        .await;
    if let Err(e) = res {
        error!("failed to get target health: {:?}", e);
        return;
    }

    let targets = res.unwrap().target_health_descriptions;
    info!("targets: {:?}", targets);

    // loop {
    //     info!("updating cache");
    //     break;
    // }
}
