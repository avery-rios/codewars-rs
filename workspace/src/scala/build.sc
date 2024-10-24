import mill._, scalalib._

object `package` extends RootModule with ScalaModule {{
  def scalaVersion = "{scala_version}"

  object sample extends ScalaTests with TestModule.ScalaTest {{
    def ivyDeps = Agg(ivy"{test_framework}")
  }}
}}
