#!/usr/bin/env ruby
# Render-test for docs/_layouts/default.html nav block.
#
# We extract the <nav> block from the layout, then render it through
# Liquid with a fake `page.url` for every nav destination. Each test
# asserts which group/link should carry class="active" for that URL.
#
# Run: ruby scripts/test_nav_render.rb
#
# Exit 0 on success, 1 on any failure. No external test framework
# needed (this is a one-off check the wiki author runs locally; CI is
# Jekyll-on-GitHub-Pages and renders the real pages).

require "liquid"

LAYOUT = File.read(File.join(__dir__, "..", "docs", "_layouts", "default.html"))

# Pull just the <nav>…</nav> block out of the layout so we don't have to
# evaluate the entire page (which references site, page.title, etc).
nav_html = LAYOUT[/<nav>.*?<\/nav>/m] or abort("could not find <nav> block")

# Inject a tiny shim for relative_url since Jekyll provides it as a
# filter but plain Liquid does not. We just pass the path through.
module RelativeUrlFilter
  def relative_url(input)
    input.to_s
  end
end
Liquid::Template.register_filter(RelativeUrlFilter)

def render(nav, url)
  tmpl = Liquid::Template.parse(nav)
  tmpl.render!("page" => { "url" => url })
end

# Parse the rendered HTML into per-group blocks and return a list of
# { trigger:, group_active:, active_child: } records — one per nav-group.
# `active_child` is the inner text of the only <a class="active"> link
# in the group, or nil if none.
def groups(rendered)
  result = []
  # Split on <div class="nav-group ...">; each piece is one group's body.
  rendered.scan(/<div class="nav-group([^"]*)">(.*?)<\/div>\s*<\/div>/m).each do |classes, body|
    trigger = body[/<button[^>]*>([^<]+)</, 1]
    # Drop nested span/caret content that snuck in (e.g., text before <span>).
    trigger = trigger&.strip
    active_child = body[/<a[^>]*class="active"[^>]*>([^<]+)</, 1]
    result << {
      trigger: trigger,
      group_active: classes.include?("active"),
      active_child: active_child,
    }
  end
  result
end

# Each case: visiting `url`, the named group's trigger should be the
# only one marked .active, and the named `child` link inside it should
# be the only .active <a> in the nav.
CASES = [
  # Worlds group
  { url: "/celestial-bodies/",                    group: "Worlds",      child: "Bodies" },
  { url: "/celestial-bodies/planets/",            group: "Worlds",      child: "Bodies" },
  { url: "/asteroid-taxonomy/",                   group: "Worlds",      child: "Asteroid Taxonomy" },
  { url: "/exoplanets/",                          group: "Worlds",      child: "Exoplanets" },
  { url: "/celestial-bodies/exoplanets.html",     group: "Worlds",      child: "Exoplanets" },
  { url: "/celestial-bodies/launch-windows.html", group: "Worlds",      child: "Launch Windows" },
  { url: "/celestial-bodies/scenario-state.html", group: "Worlds",      child: "Scenario State" },

  # Fleet
  { url: "/spacecraft/",                          group: "Fleet",       child: "Spacecraft" },
  { url: "/launch-vehicles/",                     group: "Fleet",       child: "Launch Vehicles" },

  # Build
  { url: "/facilities/",                          group: "Build",       child: "Facilities" },
  { url: "/resources/",                           group: "Build",       child: "Resources" },
  { url: "/terraforming/",                        group: "Build",       child: "Terraforming" },

  # Progression
  { url: "/research/",                            group: "Progression", child: "Research" },
  { url: "/contracts/",                           group: "Progression", child: "Contracts" },
  { url: "/missions/",                            group: "Progression", child: "Missions" },
  { url: "/achievements/",                        group: "Progression", child: "Achievements" },

  # Compare
  { url: "/corporations/",                        group: "Compare",     child: "Corporations" },
  { url: "/calculator.html",                      group: "Compare",     child: "Calculator" },

  # Home (no group should be active)
  { url: "/",                                     group: nil,           child: nil },
]

fails = 0

def fail(msg)
  warn "FAIL #{msg}"
end

CASES.each do |c|
  rendered = render(nav_html, c[:url])
  parsed = groups(rendered)

  active_groups = parsed.select { |g| g[:group_active] }
  if c[:group].nil?
    unless active_groups.empty?
      fail "url=#{c[:url].inspect} expected no active group, got #{active_groups.map { |g| g[:trigger] }.inspect}"
      fails += 1
    end
  else
    if active_groups.length != 1
      fail "url=#{c[:url].inspect} expected exactly one active group, got #{active_groups.map { |g| g[:trigger] }.inspect}"
      fails += 1
    elsif active_groups.first[:trigger] != c[:group]
      fail "url=#{c[:url].inspect} expected active group #{c[:group].inspect}, got #{active_groups.first[:trigger].inspect}"
      fails += 1
    elsif active_groups.first[:active_child] != c[:child]
      fail "url=#{c[:url].inspect} expected active child #{c[:child].inspect}, got #{active_groups.first[:active_child].inspect}"
      fails += 1
    end

    # No other group should have an active child either.
    other_active_children = parsed.reject { |g| g[:trigger] == c[:group] }.map { |g| g[:active_child] }.compact
    unless other_active_children.empty?
      fail "url=#{c[:url].inspect} unexpected active child in other group(s): #{other_active_children.inspect}"
      fails += 1
    end
  end
end

if fails == 0
  puts "OK — #{CASES.length} cases passed"
  exit 0
else
  puts "#{fails} failure(s)"
  exit 1
end
