import json
import re
import typing

from . import types


def rulesets_from_files(iterable_of_paths: typing.Iterable, ignored_providers: typing.Optional[typing.Iterable] = None) -> types.Rulesets:

    iterable_of_dicts = []

    for ruleset in iterable_of_paths:
        with open(file=ruleset, mode="r") as file:
            content = file.read()

        iterable_of_dicts.append(json.loads(content))

    rulesets = types.Rulesets()

    for ruleset in iterable_of_dicts:

        for providerName in ruleset["providers"].keys():

            if ignored_providers is not None and providerName in ignored_providers:
                continue

            # https://docs.clearurls.xyz/latest/specs/rules/#urlpattern
            urlPattern = types.Pattern(ruleset["providers"][providerName]["urlPattern"])
            urlPattern.compiled = re.compile(urlPattern)

            # https://docs.clearurls.xyz/latest/specs/rules/#completeprovider
            completeProvider = ruleset["providers"][providerName].get("completeProvider", False)

            # https://docs.clearurls.xyz/latest/specs/rules/#rules
            rules = types.Patterns()

            for rule in ruleset["providers"][providerName].get("rules", []):
                pattern = types.Pattern(rule)
                pattern.compiled = re.compile(rf"(%(?:26|23)|&|^){rule}(?:(?:=|%3[Dd])[^&]*)")

                rules.append(pattern)

            # https://docs.clearurls.xyz/latest/specs/rules/#rawrules
            rawRules = types.Patterns()

            for rawRule in ruleset["providers"][providerName].get("rawRules", []):
                pattern = types.Pattern(rawRule)
                pattern.compiled = re.compile(rawRule)

                rawRules.append(pattern)

            # https://docs.clearurls.xyz/latest/specs/rules/#referralmarketing
            referralMarketing = types.Patterns()

            for referral in ruleset["providers"][providerName].get("referralMarketing", []):
                pattern = types.Pattern(referral)
                pattern.compiled = re.compile(rf"(%(?:26|23)|&|^){referral}(?:(?:=|%3[Dd])[^&]*)")

                referralMarketing.append(pattern)

            # https://docs.clearurls.xyz/latest/specs/rules/#exceptions
            exceptions = types.Patterns()

            for exception in ruleset["providers"][providerName].get("exceptions", []):
                pattern = types.Pattern(exception)
                pattern.compiled = re.compile(exception)

                exceptions.append(pattern)

            # https://docs.clearurls.xyz/latest/specs/rules/#redirections
            redirections = types.Patterns()

            for redirection in ruleset["providers"][providerName].get("redirections", []):
                pattern = types.Pattern(redirection)
                pattern.compiled = re.compile(f"{redirection}.*")

                redirections.append(pattern)

            # https://docs.clearurls.xyz/latest/specs/rules/#forceredirection
            # This field is ignored by Unalix, we are leaving it here just as reference
            forceRedirection = ruleset["providers"][providerName].get("forceRedirection", False)

            rulesets.add_ruleset(
                types.Ruleset(
                    providerName=providerName,
                    urlPattern=urlPattern,
                    completeProvider=completeProvider,
                    rules=rules,
                    rawRules=rawRules,
                    referralMarketing=referralMarketing,
                    exceptions=exceptions,
                    redirections=redirections,
                    forceRedirection=forceRedirection
                )
            )

    return rulesets


def domains_from_files(iterable_of_paths: typing.Iterable) -> types.Domains:

    domains = types.Domains()

    for path in iterable_of_paths:
        with open(file=path, mode="r") as file:
            content = file.read()

        for domain in json.loads(content):
            domains.add_domain(domain)

    return domains


def body_redirects_from_files(iterable_of_paths: typing.Iterable) -> types.BodyRedirects:

    iterable_of_lists = []

    for path in iterable_of_paths:
        with open(file=path, mode="r") as file:
            content = file.read()

        iterable_of_lists.append(json.loads(content))

    body_redirects = types.BodyRedirects()

    for _ruleset in iterable_of_lists:

        for ruleset in _ruleset:

            providerName = ruleset["providerName"]

            if ruleset["urlPattern"] is None:
                urlPattern = None
            else:
                urlPattern = types.Pattern(ruleset["urlPattern"])
                urlPattern.compiled = re.compile(urlPattern)

            domains = types.Domains(ruleset["domains"])

            rules = types.Patterns()

            for rule in ruleset["rules"]:
                pattern = types.Pattern(rule)
                pattern.compiled = re.compile(rule)

                rules.append(pattern)

            body_redirects.add_ruleset(
                types.BodyRedirect(
                    providerName=providerName,
                    urlPattern=urlPattern,
                    domains=domains,
                    rules=rules
                )
            )

    return body_redirects
