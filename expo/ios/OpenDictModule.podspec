require 'json'

package = JSON.parse(File.read(File.join(__dir__, '..', 'package.json')))

Pod::Spec.new do |s|
  s.name           = 'OpenDictModule'
  s.version        = package['version']
  s.summary        = 'OpenDict native module for Expo'
  s.homepage       = 'https://github.com/callum/opendict-rs'
  s.license        = 'MIT'
  s.author         = 'opendict'
  s.source         = { git: '' }

  s.platform       = :ios, '15.1'
  s.swift_version  = '5.9'

  s.source_files        = '*.swift'
  s.vendored_frameworks = 'opendict_mobile.xcframework'

  s.dependency 'ExpoModulesCore'
end
