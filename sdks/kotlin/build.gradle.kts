plugins {
    kotlin("jvm") version "1.9.21"
}

group = "io.github.d4mr"
version = "0.1.0"

repositories {
    mavenCentral()
}

dependencies {
    implementation(kotlin("stdlib"))
    
    // Add JNA for FFI
    implementation("net.java.dev.jna:jna:5.13.0")
    
    testImplementation(kotlin("test"))
}

tasks.test {
    useJUnitPlatform()
}

kotlin {
    jvmToolchain(11)
}

