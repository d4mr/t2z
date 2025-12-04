plugins {
    kotlin("jvm") version "2.1.0"
    `maven-publish`
}

group = "com.d4mr"
version = "0.1.0"

repositories {
    mavenCentral()
}

dependencies {
    implementation("net.java.dev.jna:jna:5.14.0")
    
    testImplementation(kotlin("test"))
    testImplementation("org.junit.jupiter:junit-jupiter:5.10.0")
}

tasks.test {
    useJUnitPlatform()
    
    // Set library path for native library
    val libDir = file("${rootProject.projectDir}/../../crates/target/release")
    
    systemProperty("java.library.path", libDir.absolutePath)
    systemProperty("jna.library.path", libDir.absolutePath)
    
    // For macOS
    environment("DYLD_LIBRARY_PATH", libDir.absolutePath)
    // For Linux
    environment("LD_LIBRARY_PATH", libDir.absolutePath)
}

// Force JVM 21 target for both Java and Kotlin (Kotlin doesn't support 25 yet)
java {
    sourceCompatibility = JavaVersion.VERSION_21
    targetCompatibility = JavaVersion.VERSION_21
}

tasks.withType<org.jetbrains.kotlin.gradle.tasks.KotlinCompile> {
    kotlinOptions {
        jvmTarget = "21"
    }
}

publishing {
    publications {
        create<MavenPublication>("maven") {
            groupId = "com.d4mr"
            artifactId = "t2z"
            version = project.version.toString()
            
            from(components["java"])
            
            pom {
                name.set("t2z")
                description.set("Kotlin SDK for transparent → shielded Zcash transactions")
                url.set("https://github.com/d4mr/t2z")
                
                licenses {
                    license {
                        name.set("MIT License")
                        url.set("https://opensource.org/licenses/MIT")
                    }
                }
                
                developers {
                    developer {
                        id.set("d4mr")
                        name.set("d4mr")
                    }
                }
                
                scm {
                    url.set("https://github.com/d4mr/t2z")
                }
            }
        }
    }
}
