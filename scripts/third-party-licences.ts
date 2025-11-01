import fs from 'node:fs'
import Handlebars from 'handlebars'
import { execSync } from 'node:child_process'

interface Package {
    name: string
    licenses: string[]
    rawLicense: string
    version: string
    url: string
    type: 'NPM' | 'Cargo'
}

interface CargoPackage {
    package: string
    repository_url: string
    license: string
}

interface NpmPackage {
    licenses: string
    repository: string
    publisher: string
    email: string
    path: string
    licenseFile: string
}

type CargoPackages = CargoPackage[]
type NpmPackages = Record<string, NpmPackage>;

export interface LicenseInfo {
    reference: string
    isDeprecatedLicenseId: boolean
    detailsUrl: string
    referenceNumber: number
    name: string
    licenseId: string
    seeAlso: string[]
    isOsiApproved: boolean
}

type LicensesInfo = Record<string, {
    name: string,
    url: string
}>

if (!fs.existsSync('.cache')) {
    fs.mkdirSync('.cache')
}

const cargoPackagesJson = execSync('cargo tree --all-features --prefix=none --format=\'{{"package":"{p}","repository_url":"{r}","license":"{l}"}}\'')
    .toString()
    .split(/\n|\(\*\)/)
    .filter(line => line.trim() !== '')
    .join(',')
const npmPackagesJson = execSync('npm run show:package-licenses --silent').toString()
const cargoPackages: CargoPackages = JSON.parse(`[${cargoPackagesJson}]`)
const npmPackages: NpmPackages = JSON.parse(npmPackagesJson)

const fetchLicensesInfo = async (): Promise<LicensesInfo> => {
    if (fs.existsSync('.cache/licenses.json')) {
        return JSON.parse(fs.readFileSync('.cache/licenses.json', 'utf-8'))
    }

    const licensesInfo: LicenseInfo[] = await fetch('https://raw.githubusercontent.com/spdx/license-list-data/refs/heads/main/json/licenses.json')
        .then(response => response.json())
        .then(data => data.licenses)
        .catch(error => {
            console.error('Error fetching licenses:', error)
            return []
        })

    const licensesWithValidUrls = licensesInfo.map(async license => {
        const urlsStatus: [string, Response][] = await Promise.all(
            license.seeAlso
                .filter(url => /^https?:\/\//.test(url))
                .map(async (url) => [url, await fetch(url).catch(() => ({ status: 404 }))] as [string, Response])
        )
        const urls = urlsStatus
            // eslint-disable-next-line @typescript-eslint/no-unused-vars
            .filter(([_, response]) => response.status === 200)
            // eslint-disable-next-line @typescript-eslint/no-unused-vars
            .map(([url, _]) => url)

        const info = {
            name: license.name,
            url: urls[0]
        }

        return [license.licenseId, info]
    })

    const availableLicenses = Object.fromEntries(
        await Promise.all(licensesWithValidUrls)
    )

    fs.writeFileSync('.cache/licenses.json', JSON.stringify(availableLicenses))
    return availableLicenses
}

const licenses = await fetchLicensesInfo()

const extractLicenses = (licenses: string): string[] => {
    const licenseRegex = / *(\+|OR|WITH|AND|\/) */
    const filterLicenses = (license: string): boolean => {
        const isSeparator = licenseRegex.test(license)
        return license !== ''
            && license !== '/'
            && !isSeparator
            && !license.includes('-exception')
    }

    return licenses
        .replace(/[()]/, '')
        .split(licenseRegex)
        .map(license => license.trim().replace(/[()]/, ''))
        .filter(filterLicenses)
}

const mapCargoPackage = (cargoPackage: CargoPackage): Package => {
    const info = cargoPackage.package.split(' ')

    return {
        name: info[0],
        version: info[1],
        url: cargoPackage.repository_url,
        licenses: extractLicenses(cargoPackage.license),
        rawLicense: cargoPackage.license,
        type: 'Cargo'
    }
}

const mapNpmPackage = ([packageName, packageInfo]: [string, NpmPackage]): Package | null => {
    const regex = /^(@?.+\/?)@(.+)$/
    const groups = regex.exec(packageName)
    if (!groups) {
        console.warn(`unable to extract package name and version from: ${packageName}`)
        return null
    }

    return {
        name: groups[1],
        version: groups[2],
        licenses: extractLicenses(packageInfo.licenses),
        rawLicense: packageInfo.licenses,
        url: packageInfo.repository,
        type: 'NPM'
    }
}

const allPackages: Package[] = cargoPackages
    .map(mapCargoPackage)
    .concat(
        Object.entries(npmPackages)
            .map(mapNpmPackage)
            .filter(item => item != null)
    )
    .filter(item => item.licenses.length > 0 && !item.name.includes('mikupush') && !item.name.includes('miku-push'))

const packagesByLicense: Record<string, Package[]> = {}

for (const item of allPackages) {
    for (const license of item.licenses) {
        const packages = packagesByLicense[license] ?? []
        packages.push(item)
        packagesByLicense[license] = packages
    }
}

const packageLicenses = Object.keys(packagesByLicense)

const template = `
# Third-party licenses

Miku Push! Server is built using several open-source libraries, runtimes, and frameworks. You can find their licenses listed below, organized by license.

There are packages with multiple licenses. The licenses are listed in the order they appear in the package.

## Licences of all packages

{{#each packageLicenses}}
- [{{ this }}](#{{anchor this }})
{{/each}}

{{#each packagesByLicense}}
### {{ @key }}

{{seeLicense @key}}

{{#each this}}
#### {{ name }}

- **Version**: {{ version }}
- **Description**: {{ description }}
- **URL**: [{{ url }}]({{ url }})
- **License**: {{ rawLicense }}
- **Type**: {{ type }}
{{/each}}
{{/each}}
`

Handlebars.registerHelper('anchor', (value: string) => {
    return value.replace(/\./g, '').toLowerCase()
})

Handlebars.registerHelper('seeLicense', (value: string) => {
    const name = licenses[value]?.name ?? value
    const url = licenses[value]?.url
    if (!url) {
        return ''
    }

    return `[See ${name}](${url})`
})

const compiledTemplate = Handlebars.compile(template)
const output = compiledTemplate({ packageLicenses, packagesByLicense, licenses })

fs.writeFileSync('THIRD_PARTY_LICENSES.md', output)
