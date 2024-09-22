resource "aws_route53_record" "aelred" {
  type    = local.apex ? "A" : "CNAME"
  name    = local.full_domain
  zone_id = data.aws_route53_zone.zone.zone_id
  ttl     = 43200
  records = local.apex ? local.github_pages_ips : local.github_pages_urls
}


resource "github_repository" "this" {
  name         = var.repo == null ? local.full_domain : var.repo
  homepage_url = "https://${local.full_domain}"

  pages {
    build_type = "workflow"
    cname      = local.full_domain
  }

  lifecycle {
    prevent_destroy = true
    ignore_changes = [
      auto_init, has_issues, has_downloads, has_projects, has_wiki, is_template, vulnerability_alerts, topics,
      description
    ]
  }
}

data "github_user" "me" {
  username = ""
}

data "aws_route53_zone" "zone" {
  name = var.domain
}

locals {
  apex              = var.subdomain == null
  full_domain       = var.subdomain == null ? var.domain : "${var.subdomain}.${var.domain}"
  github_pages_urls = ["${data.github_user.me.login}.github.io."]
  github_pages_ips  = ["185.199.108.153", "185.199.109.153", "185.199.110.153", "185.199.111.153"]
}
