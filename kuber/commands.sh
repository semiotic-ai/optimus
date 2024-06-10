gcloud iam service-accounts add-iam-policy-binding buckets-service@graphplots.iam.gserviceaccount.com \
    --role roles/iam.workloadIdentityUser \
    --member "serviceAccount:graphplots.svc.id.goog[substreams/default]"


kubectl annotate serviceaccount default \
    --namespace substreams \
    iam.gke.io/gcp-service-account=buckets-service@graphplots.iam.gserviceaccount.com


 gcloud storage cp substreams.spkg gs://semiotic-sinker/substreams/erc20.spkg


 