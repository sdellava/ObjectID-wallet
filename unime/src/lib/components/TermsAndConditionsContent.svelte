<script lang="ts">
  const termsUrl = 'https://objectid.io/general-terms/';

  const sections = [
    {
      title: '1. Definitions',
      items: [
        '"Website" refers to ObjectID.io and all its subdomains and services.',
        '"Service" refers to any functionality provided by ObjectID.io, including APIs, user interfaces, smart contract interactions, and credit-based operations.',
        '"Credits" are consumable, non-transferable units required to use ObjectID services.',
        '"User", "You", or "Client" refers to any authorized person or entity accessing or using the Service.',
        '"We", "Us", or "Our" refers to SDV Consulting SRLS.',
        '"dApp" refers to a distributed application that does not rely on a centralized server and/or database, but interacts directly with a smart contract on a public blockchain.',
      ],
    },
    {
      title: '2. Authorized Access and Credit-Based Use',
      body: 'Use of ObjectID is strictly reserved to authorized users who have obtained non-transferable Credits issued by SDV Consulting SRLS.',
      items: [
        'Credits are required to interact with the ObjectID infrastructure, for example to create or update objects.',
        'Credits cannot be transferred, resold, or exchanged between users unless explicitly authorized in writing by SDV Consulting.',
        'Purchasing Credits does not grant rights over ObjectID trademarks, logos, software code, or proprietary technologies.',
      ],
    },
    {
      title: '2-bis. ObjectID Storage Service',
      body: 'Use of the ObjectID Storage service available through storage.objectid.io is subject to these Terms and to the ObjectID Storage Service Addendum. ObjectID Storage relies on IPFS-oriented workflows and may submit storage, pinning, verification, retention, renewal, or related tasks to the IOTA Distributed Oracle. The User remains solely responsible for uploaded files, task configuration, retention choices, legal compliance, and ensuring that no unlawful, infringing, malicious, unauthorized, or improperly processed personal data is uploaded or referenced through the Storage service.',
    },
    {
      title: '3. Intellectual Property',
      body: 'All contents of the Website, including text, images, code, and trademarks, are the exclusive property of SDV Consulting SRLS or its licensors. No right of reproduction or reuse is granted except where explicitly stated.',
    },
    {
      title: '4. Service Modifications',
      body: 'We reserve the right to modify or discontinue any aspect of the Website, Service, or pricing structure at any time without prior notice.',
    },
    {
      title: '5. Public Blockchain Disclaimer',
      body: 'ObjectID is built on a public blockchain infrastructure.',
      items: [
        'All submitted data becomes public and immutable.',
        'The User is solely responsible for ensuring that any data submitted complies with GDPR, MiCAR, and other applicable laws and regulations.',
        'SDV Consulting SRLS cannot alter or delete data once submitted to the blockchain.',
      ],
    },
    {
      title: '6. Decentralized Application Disclaimer',
      body: 'The User understands and accepts that ObjectID is a decentralized application interacting directly with blockchain smart contracts. SDV Consulting SRLS does not act as an intermediary in transactions between the user and the smart contract, has no control over the blockchain, and cannot guarantee its availability, correctness, or immutability. The User waives any claim against SDV Consulting SRLS for damages or losses resulting from interaction with ObjectID smart contracts, misuse or misinterpretation of the service, technical anomalies, bugs, or blockchain malfunctions.',
    },
    {
      title: '7. Data Privacy',
      body: 'SDV Consulting SRLS complies with the EU General Data Protection Regulation (GDPR). For more information, please refer to the Privacy Policy published by ObjectID.',
    },
    {
      title: '8. Limitation of Liability',
      body: 'To the fullest extent permitted by law, SDV Consulting SRLS is not liable for any loss or damage, including data loss, service interruption, or smart contract behavior, arising from the use or inability to use the ObjectID service. No warranty is provided regarding the availability, security, or accuracy of the service.',
    },
    {
      title: '9. Governing Law and Jurisdiction',
      body: 'These Terms shall be governed by and construed in accordance with the laws of Italy. Any disputes shall be exclusively subject to the Court of Monza and Brianza, Italy.',
    },
    {
      title: '10. Contact',
      body: 'For questions about these Terms, please contact SDV Consulting SRLS, via della Conciliazione 13, 20862 Arcore (MB), Italy, or email info@objectid.io.',
    },
  ];
</script>

<div class="space-y-4 text-[12px]/[20px] font-medium text-slate-600 dark:text-slate-300">
  <div class="rounded-lg border border-slate-200 bg-white p-4 dark:border-slate-600 dark:bg-dark">
    <h3 class="text-base font-semibold text-slate-900 dark:text-grey">ObjectID General Terms and Conditions</h3>
    <p class="pt-1 text-slate-500 dark:text-slate-300">Effective Date: 1 June 2025</p>
    <p class="pt-3">
      Welcome to ObjectID.io, a service provided by SDV Consulting SRLS, via della Conciliazione 13, 20862 Arcore (MB),
      Italy (VAT IT13168650961). By accessing or using this website, dApp, APIs, smart-contract interactions,
      credit-based operations, or associated services, you agree to be bound by these terms and conditions.
    </p>
    <a
      class="mt-3 inline-flex font-semibold text-black underline dark:text-white"
      href={termsUrl}
      target="_blank"
      rel="noreferrer"
    >
      Open latest version on objectid.io
    </a>
  </div>

  {#each sections as section}
    <section class="rounded-lg border border-slate-200 bg-white p-4 dark:border-slate-600 dark:bg-dark">
      <h4 class="text-sm font-semibold text-slate-900 dark:text-grey">{section.title}</h4>
      {#if section.body}
        <p class="pt-2">{section.body}</p>
      {/if}
      {#if section.items}
        <ul class="list-disc space-y-2 pt-2 pl-5">
          {#each section.items as item}
            <li>{item}</li>
          {/each}
        </ul>
      {/if}
    </section>
  {/each}
</div>
