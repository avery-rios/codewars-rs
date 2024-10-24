plugins {{
    kotlin("jvm") version("{kotlin_version}")
}}

repositories {{
    mavenCentral()
}}

dependencies {{
    testImplementation(kotlin("test"))
}}
