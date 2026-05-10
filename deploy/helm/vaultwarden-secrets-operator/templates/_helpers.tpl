{{/*
Expand the name of the chart.
*/}}
{{- define "vwso.name" -}}
{{- default .Chart.Name .Values.nameOverride | trunc 63 | trimSuffix "-" -}}
{{- end -}}

{{/*
Create a default fully qualified app name.
*/}}
{{- define "vwso.fullname" -}}
{{- if .Values.fullnameOverride -}}
{{- .Values.fullnameOverride | trunc 63 | trimSuffix "-" -}}
{{- else -}}
{{- $name := default .Chart.Name .Values.nameOverride -}}
{{- if contains $name .Release.Name -}}
{{- .Release.Name | trunc 63 | trimSuffix "-" -}}
{{- else -}}
{{- printf "%s-%s" .Release.Name $name | trunc 63 | trimSuffix "-" -}}
{{- end -}}
{{- end -}}
{{- end -}}

{{/*
Create chart name and version as used by the chart label.
*/}}
{{- define "vwso.chart" -}}
{{- printf "%s-%s" .Chart.Name .Chart.Version | replace "+" "_" | trunc 63 | trimSuffix "-" -}}
{{- end -}}

{{/*
Common labels.
*/}}
{{- define "vwso.labels" -}}
helm.sh/chart: {{ include "vwso.chart" . }}
{{ include "vwso.selectorLabels" . }}
app.kubernetes.io/version: {{ .Chart.AppVersion | quote }}
app.kubernetes.io/managed-by: {{ .Release.Service }}
{{- end -}}

{{/*
Selector labels.
*/}}
{{- define "vwso.selectorLabels" -}}
app.kubernetes.io/name: {{ include "vwso.name" . }}
app.kubernetes.io/instance: {{ .Release.Name }}
{{- end -}}

{{/*
Service account name.
*/}}
{{- define "vwso.serviceAccountName" -}}
{{- if .Values.serviceAccount.create -}}
{{- default (include "vwso.fullname" .) .Values.serviceAccount.name -}}
{{- else -}}
{{- default "default" .Values.serviceAccount.name -}}
{{- end -}}
{{- end -}}

{{/*
Credentials Secret name.
*/}}
{{- define "vwso.credentialsSecretName" -}}
{{- if .Values.credentials.existingSecret.name -}}
{{- .Values.credentials.existingSecret.name -}}
{{- else -}}
{{- printf "%s-credentials" (include "vwso.fullname" .) -}}
{{- end -}}
{{- end -}}

{{/*
Validate endpoint and credential configuration.
*/}}
{{- define "vwso.validate" -}}
{{- if and .Values.config.vaultwardenUrl (or .Values.config.identityUrl .Values.config.apiUrl) -}}
{{- fail "configure either config.vaultwardenUrl or both config.identityUrl and config.apiUrl, not both endpoint modes" -}}
{{- end -}}
{{- if and (not .Values.config.vaultwardenUrl) (not (and .Values.config.identityUrl .Values.config.apiUrl)) -}}
{{- fail "configure config.vaultwardenUrl, or both config.identityUrl and config.apiUrl" -}}
{{- end -}}
{{- if and .Values.credentials.create .Values.credentials.existingSecret.name -}}
{{- fail "configure either credentials.create or credentials.existingSecret.name, not both" -}}
{{- end -}}
{{- if and (not .Values.credentials.create) (not .Values.credentials.existingSecret.name) -}}
{{- fail "configure credentials.existingSecret.name or set credentials.create=true" -}}
{{- end -}}
{{- if .Values.credentials.create -}}
{{- if not .Values.credentials.clientId -}}
{{- fail "credentials.clientId is required when credentials.create=true" -}}
{{- end -}}
{{- if not .Values.credentials.clientSecret -}}
{{- fail "credentials.clientSecret is required when credentials.create=true" -}}
{{- end -}}
{{- if not .Values.credentials.masterPassword -}}
{{- fail "credentials.masterPassword is required when credentials.create=true" -}}
{{- end -}}
{{- end -}}
{{- end -}}
