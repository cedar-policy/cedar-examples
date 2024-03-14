import matplotlib.pyplot as plt
import pandas
import os
import pdb

# we'd love to use str.removeprefix() but it only exists in Python 3.9+ and AL2 is old
# so we have this instead
# https://stackoverflow.com/questions/16891340/remove-a-prefix-from-a-string
def removeprefix(text, prefix):
    if text.startswith(prefix):
        return text[len(prefix):]
    return text

def append_before_first_space(text, to_append):
    index = text.index(' ')
    return text[:index] + to_append + text[index:]

class IndependentVar:
    def __init__(self, data_name, pretty_name):
        self.data_name = data_name
        self.pretty_name = pretty_name

num_entities = IndependentVar("num_entities", "Number of entities per entity type")
unstructured_bytes = IndependentVar("unstructured_bytes", "Number of bytes in `Unstructured`")

class Engine:
    def __init__(self, data_name, pretty_name, median_style, p99_style):
        self.data_name = data_name
        self.pretty_name = pretty_name
        self.median_style = median_style
        self.p99_style = p99_style

    def __repr__(self):
        return self.pretty_name

    def plot_times(self, data, independent_var=num_entities):
        """
        Perform just the `plt.plot` calls for this engine on this data.
        Doesn't initialize or finalize the plot, so that you can add other lines
        (eg from other calls to `plot_times()`) if you want

        `data` should be a `DataSource`
        """
        if data is None or data.data is None or data.data.empty:
            return None
        median_data_name = '%s median_dur_micros' % self.data_name
        p99_data_name = '%s p99_dur_micros' % self.data_name
        median_label = '%s median' % self.pretty_name
        p99_label = '%s p99' % self.pretty_name
        if median_data_name in data.data:
            plt.plot(independent_var.data_name, median_data_name, self.median_style, data=data.data, label=median_label)
        if p99_data_name in data.data:
            plt.plot(independent_var.data_name, p99_data_name, self.p99_style, data=data.data, label=p99_label)

    def report_avg_median(self, data):
        """
        Return the average median time, in micros, across all tested inputs

        `data` should be a `DataSource`
        """
        col_name = '%s median_dur_micros' % self.data_name
        if data is not None and data.data is not None and not data.data.empty and col_name in data.data:
            return data.data[col_name].mean()
        else:
            return None

class DataSource:
    def __init__(self, filename, pretty_name):
        self.pretty_name = pretty_name
        try:
            with open(filename, newline='') as file:
                self.data = pandas.read_csv(file, sep=',', quotechar='"')
        except FileNotFoundError:
            self.data = None

cedar = Engine('cedar', 'Cedar', 'o-g', 'o--g')
cedar_templates = Engine('cedar', 'Cedar (Templates)', 'o-C1', 'o--C1')
cedar_opt = Engine('cedaropt', 'Cedar (Policy Slicing)', 'o-C4', 'o--C4')
openfga = Engine('openfga', 'OpenFGA', 'v-C1', 'v--C1')
openfga_templates = Engine('openfga', 'OpenFGA (Templates)', 'v-C3', 'v--C3')
rego = Engine('rego', 'Rego', 's-C0', 's--C0')
rego_pre_tc = Engine('rego_pre_tc', 'Rego (Pre-TC)', 's-C3', 's--C3')
opa = Engine('opa', 'Rego', 's-C0', 's--C0')
opa_tc = Engine('opa_tco', 'Rego (TC)', 's-C3', 's--C3')

def plot_times(engines_with_data, folder, independent_var=num_entities, small=False, omit_ylabel=False, xlabel_fontsize=None, ylabel_fontsize=None):
    """
    engines_with_data: list or iterable of (engine, data) pairs
        (where the engine is an `Engine` and data is a `DataSource`)

    small: if `True`, the generated figure will be somewhat smaller

    xlabel_fontsize and ylabel_fontsize: if `None`, use the pyplot default. Otherwise, accepted values are:
        'xx-small', 'x-small', 'small', 'medium', 'large', 'x-large', 'xx-large'
        or any float number
    """

    plt.figure(clear=True, figsize=[4.8, 3.6] if small else [6.4, 4.8])
    print("Average medians across all tested inputs:")
    for (engine, data) in engines_with_data:
        engine.plot_times(data, independent_var=independent_var)
        avg_median = engine.report_avg_median(data)
        if avg_median is not None:
            print(f'{data.pretty_name} with {engine.pretty_name}: {avg_median:.1f} micros')

    plt.ylim(bottom = 0)
    if xlabel_fontsize is not None:
        plt.xlabel(independent_var.pretty_name, fontsize=xlabel_fontsize)
    else:
        plt.xlabel(independent_var.pretty_name)
    if not omit_ylabel:
        if ylabel_fontsize is not None:
            plt.ylabel('authorization time (µs)', fontsize=ylabel_fontsize)
        else:
            plt.ylabel('authorization time (µs)')
    plt.legend()
    plt.tight_layout()
    try:
        os.mkdir(folder)
    except FileExistsError:
        pass
    plt.savefig(folder + '/times_vs_' + independent_var.data_name + '.pdf')
    plt.close()

def plot_parents(data, folder, independent_var=num_entities):
    plt.figure(clear=True)
    for et in data.data.columns:
        if et.startswith("mean_parents_of"):
            plt.plot(independent_var.data_name, et, 'o-', data=data.data, label=removeprefix(et, 'mean_parents_of_'))
    plt.xlabel(independent_var.pretty_name)
    plt.ylabel('Mean number of parents for entities of the given type')
    plt.legend()
    try:
        os.mkdir(folder)
    except FileExistsError:
        pass
    plt.savefig(folder + '/parents_vs_' + independent_var.data_name + '.pdf')
    plt.close()

def plot_allows_denies(data, folder, independent_var=num_entities):
    plt.figure(clear=True)
    if r'cedar % allows' in data.data:
        label = r'Cedar % allows'
        plt.plot(independent_var.data_name, r'cedar % allows', cedar.median_style, data=data.data, label=label)
    if r'cedar % denies' in data.data:
        label = r'Cedar % denies'
        plt.plot(independent_var.data_name, r'cedar % denies', cedar.median_style, data=data.data, label=label)
    if r'openfga % allows' in data.data:
        plt.plot(independent_var.data_name, r'openfga % allows', openfga.median_style, data=data.data, label=r'OpenFGA % allows')
    if r'openfga % denies' in data.data:
        plt.plot(independent_var.data_name, r'openfga % denies', openfga.median_style, data=data.data, label=r'OpenFGA % denies')
    plt.ylim(0, 1)
    plt.xlabel(independent_var.pretty_name)
    plt.ylabel('Percentage of requests')
    plt.legend()
    try:
        os.mkdir(folder)
    except FileExistsError:
        pass
    plt.savefig(folder + '/allowsdenies_vs_' + independent_var.data_name + '.pdf')
    plt.close()

def plot_openfga_tuples(data, folder, independent_var=num_entities, alt_data=None, alt_label='(Templates)'):
    plt.figure(clear=True)
    plotted = False
    if 'openfga mean_tuples' in data.data:
        plotted = True
        plt.plot(independent_var.data_name, 'openfga mean_tuples', openfga.median_style, data=data.data, label='OpenFGA num_tuples')
    if alt_data is not None and alt_data.data is not None and 'openfga mean_tuples' in alt_data.data:
        plotted = True
        plt.plot(independent_var.data_name, 'openfga mean_tuples', openfga_templates.median_style, data=alt_data.data, label='OpenFGA ' + alt_label + ' num_tuples')
    if plotted:
        plt.ylim(bottom = 0)
        plt.xlabel(independent_var.pretty_name)
        plt.ylabel('Number of OpenFGA tuples added')
        plt.legend()
        try:
            os.mkdir(folder)
        except FileExistsError:
            pass
        plt.savefig(folder + '/num_tuples_vs_' + independent_var.data_name + '.pdf')
    plt.close()

github_data = DataSource("output/github.csv", "github")
github_templates_data = DataSource("output/github-templates.csv", "github-templates")
gdrive_data = DataSource("output/gdrive.csv", "gdrive")
gdrive_templates_data = DataSource("output/gdrive-templates.csv", "gdrive-templates")
tinytodo_data = DataSource("output/tinytodo.csv", "tinytodo")

if github_data.data is not None:
    github_num_entities = IndependentVar("num_entities", "Number of Users, Teams, Repos, and Orgs")
    plot_times(
        (
            (cedar, github_data),
            #(cedar_templates, github_templates_data),
            (openfga, github_data),
            #(openfga_templates, github_templates_data),
            (rego, github_data),
            (rego_pre_tc, github_data),
        ),
        'output/github',
        github_num_entities,
        small=True,
        omit_ylabel=False,
        xlabel_fontsize='large',
    )
    plot_parents(github_data, 'output/github', github_num_entities)
    plot_allows_denies(github_data, 'output/github', github_num_entities)
    plot_openfga_tuples(github_data, 'output/github', github_num_entities, alt_data=github_templates_data, alt_label="(Templates)")
    if github_templates_data.data is not None:
        plot_times(
            (
                (cedar, github_data),
                (cedar_templates, github_templates_data),
                (cedar_opt, github_templates_data),
            ),
            'output/github-slicing',
            github_num_entities,
            small=True,
        )

if gdrive_data.data is not None:
    gdrive_num_entities = IndependentVar("num_entities", "Number of Users, Groups, Documents, and Folders")
    plot_times(
        (
            (cedar, gdrive_data),
            #(cedar_templates, gdrive_templates_data),
            (openfga, gdrive_data),
            #(openfga_templates, gdrive_templates_data),
            (rego, gdrive_data),
            (rego_pre_tc, gdrive_data),
        ),
        'output/gdrive',
        gdrive_num_entities,
        small=True,
        xlabel_fontsize='large',
        ylabel_fontsize='large',
    )
    plot_parents(gdrive_data, 'output/gdrive', gdrive_num_entities)
    plot_allows_denies(gdrive_data, 'output/gdrive', gdrive_num_entities)
    plot_openfga_tuples(gdrive_data, 'output/gdrive', gdrive_num_entities, alt_data=gdrive_templates_data, alt_label="(Templates)")
    if gdrive_templates_data.data is not None:
        plot_times(
            (
                (cedar, gdrive_data),
                (cedar_templates, gdrive_templates_data),
                (cedar_opt, gdrive_templates_data),
            ),
            'output/gdrive-slicing',
            gdrive_num_entities,
            small=True,
        )

if tinytodo_data.data is not None:
    tinytodo_num_entities = IndependentVar("num_entities", "Number of Users, Teams, and Lists")
    plot_times(
        (
            (cedar, tinytodo_data),
            (openfga, tinytodo_data),
            (rego, tinytodo_data),
            (rego_pre_tc, tinytodo_data),
        ),
        'output/tinytodo',
        tinytodo_num_entities,
        small=True,
        omit_ylabel=False,
        xlabel_fontsize='large',
    )
    plot_parents(tinytodo_data, 'output/tinytodo', tinytodo_num_entities)
    plot_allows_denies(tinytodo_data, 'output/tinytodo', tinytodo_num_entities)
    plot_openfga_tuples(tinytodo_data, 'output/tinytodo', tinytodo_num_entities)
