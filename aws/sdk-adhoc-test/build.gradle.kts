/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

extra["displayName"] = "Smithy :: Rust :: AWS-SDK :: Ad-hoc Test"
extra["moduleName"] = "software.amazon.smithy.rust.awssdk.adhoc.test"

tasks["jar"].enabled = false

plugins {
    java
    id("software.amazon.smithy.gradle.smithy-base")
    id("software.amazon.smithy.gradle.smithy-jar")
}

java {
    sourceCompatibility = JavaVersion.VERSION_11
    targetCompatibility = JavaVersion.VERSION_11
}

val smithyVersion: String by project
val defaultRustDocFlags: String by project
val properties = PropertyRetriever(rootProject, project)

val pluginName = "rust-client-codegen"
val workingDirUnderBuildDir = "smithyprojections/sdk-adhoc-test/"

configure<software.amazon.smithy.gradle.SmithyExtension> {
    outputDirectory = layout.buildDirectory.dir(workingDirUnderBuildDir).get().asFile
}

dependencies {
    implementation(project(":aws:sdk-codegen"))
    implementation("software.amazon.smithy:smithy-aws-protocol-tests:$smithyVersion")
    implementation("software.amazon.smithy:smithy-protocol-test-traits:$smithyVersion")
    implementation("software.amazon.smithy:smithy-aws-traits:$smithyVersion")
    implementation("software.amazon.smithy:smithy-model:$smithyVersion")
}

fun getNullabilityCheckMode(): String = properties.get("nullability.check.mode") ?: "CLIENT_CAREFUL"

fun baseTest(service: String, module: String, imports: List<String> = listOf()): CodegenTest {
    return CodegenTest(
        service = service,
        module = module,
        imports = imports,
        extraCodegenConfig = """
            "includeFluentClient": false,
            "nullabilityCheckMode": "${getNullabilityCheckMode()}"
        """,
    )
}

val allCodegenTests = listOf(
    baseTest(
        "com.amazonaws.apigateway#BackplaneControlService",
        "apigateway",
        imports = listOf("models/apigateway-rules.smithy"),
    ),
    baseTest(
        "com.amazonaws.testservice#TestService",
        "endpoint-test-service",
        imports = listOf("models/single-static-endpoint.smithy"),
    ),
    baseTest(
        "com.amazonaws.testservice#RequiredValues",
        "required-values",
        imports = listOf("models/required-value-test.smithy"),
    ),
)

project.registerGenerateSmithyBuildTask(rootProject, pluginName, allCodegenTests)
project.registerGenerateCargoWorkspaceTask(rootProject, pluginName, allCodegenTests, workingDirUnderBuildDir)
project.registerGenerateCargoConfigTomlTask(layout.buildDirectory.dir(workingDirUnderBuildDir).get().asFile)

tasks["jar"].dependsOn("generateSmithyBuild")
tasks["assemble"].finalizedBy("generateCargoWorkspace")

project.registerModifyMtimeTask()
project.registerCargoCommandsTasks(layout.buildDirectory.dir(workingDirUnderBuildDir).get().asFile, defaultRustDocFlags)

tasks["test"].finalizedBy(cargoCommands(properties).map { it.toString })

tasks["clean"].doFirst { delete("smithy-build.json") }
